use crate::common::Error::NoSuchFile;
use crate::common::{json, ContentType, Error, Res};
use bytes::Bytes;
use futures_core::Stream;
use serde::de;
use std::cell::RefCell;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::ops::Add;
use std::path::Path;
use std::pin::Pin;
use std::sync::{Arc, Mutex, RwLock};
use std::task::Poll::Ready;
use std::task::{Context, Poll};
use log::debug;

static FILE_SEPARATOR: &str = if cfg!(target_os = "windows") {
    "\\"
} else {
    "/"
};

macro_rules! static_root_volume {
    ($name:ident, $value:expr) => {
        #[allow(non_snake_case, missing_docs, dead_code)]
        pub fn $name() -> DirNode {
            DirNode {
                node: Arc::new(RwLock::new((String::from($value), None))),
            }
        }
    };
}

macro_rules! static_file_context_type {
    ($id:expr, $name:ident, $file_type:expr, $ext:expr, $mimetype:expr, $magic:expr) => {
        pub const $name: ContentType = ContentType {
            id: $id,
            file: $file_type,
            ext: $ext,
            mimetype: $mimetype,
            magic: $magic,
        };
    };
}

pub trait Node {
    fn name(&self) -> String;

    fn up(&self) -> Option<DirNode>;

    fn set_parent(&mut self, parent: &DirNode);

    fn set_root_parent(&mut self, parent: &DirNode) {
        match self.up() {
            None => self.set_parent(parent),
            Some(mut up) => up.set_root_parent(parent),
        }
    }

    fn set_name(&mut self, new_name: String);

    fn from(path: &str) -> Self;

    fn absolute_path(&self) -> String {
        if let Some(parent) = &self.up() {
            parent.absolute_path().add(FILE_SEPARATOR).add(&self.name())
        } else {
            String::from(self.name())
        }
    }

    fn deep(&self) -> i16 {
        if let Some(parent) = &self.up() {
            parent.deep() + 1
        } else {
            0
        }
    }

    fn is_exist(&self) -> bool {
        Path::new(&self.absolute_path()).exists()
    }

    fn move_to(&mut self, parent: &DirNode) -> Res<()> {
        let old_path = self.absolute_path();
        self.set_parent(parent);
        let new_path = self.absolute_path();
        fs::rename(&old_path, &new_path).map_err(warp_e)
    }

    fn rename(&mut self, new_name: String) -> Res<()> {
        let old_path = self.absolute_path();
        self.set_name(new_name);
        let new_path = self.absolute_path();
        if self.is_exist() {
            match fs::rename(old_path, new_path) {
                Ok(_) => Ok(()),
                Err(e) => Err(Error::DirectoryError(e.to_string())),
            }
        } else {
            Ok(())
        }
    }
}

pub struct DirNode {
    node: Arc<RwLock<(String, Option<DirNode>)>>,
}

pub struct FileNode {
    node: Arc<RwLock<(String, Option<DirNode>)>>,
    content_type: &'static ContentType,
    file: Arc<RefCell<Option<File>>>,
}

impl Node for DirNode {
    fn name(&self) -> String {
        String::from(&self.node.read().unwrap().0)
    }

    fn up(&self) -> Option<DirNode> {
        self.node.read().unwrap().1.clone()
    }

    fn set_parent(&mut self, parent: &DirNode) {
        self.node.write().unwrap().1 = Some(parent.clone())
    }

    fn set_name(&mut self, new_name: String) {
        self.node.write().unwrap().0 = new_name
    }

    fn from(path: &str) -> Self {
        let node_name_list = path.split(FILE_SEPARATOR);
        let mut parent = None;
        for node_name in node_name_list {
            parent = Some(DirNode {
                node: Arc::new(RwLock::new((node_name.to_string(), parent))),
            })
        }
        parent.unwrap()
    }
}

impl Node for FileNode {
    fn name(&self) -> String {
        format!("{}{}", self.node.read().unwrap().0, self.content_type.ext)
    }

    fn up(&self) -> Option<DirNode> {
        self.node.read().unwrap().1.clone()
    }

    fn set_parent(&mut self, parent: &DirNode) {
        self.node.write().unwrap().1 = Some(parent.clone())
    }

    fn set_name(&mut self, new_name: String) {
        self.node.write().unwrap().0 = new_name
    }

    fn from(path: &str) -> Self {
        let node_name_list = path.split(FILE_SEPARATOR);
        let mut node_name_vec: Vec<_> = node_name_list.collect();
        let finial = node_name_vec.pop();

        let mut parent = None;
        for node_name in node_name_vec {
            parent = Some(DirNode {
                node: Arc::new(RwLock::new((node_name.to_string(), parent))),
            })
        }
        let file_split = finial.unwrap().split(".");
        let file_name_vec = file_split.collect::<Vec<&str>>();
        let file_name = file_name_vec.get(0).unwrap();
        let ext = file_name_vec
            .get(1)
            .map(|ext| format!(".{}", ext))
            .map(|ext| content_type::guess(&ext))
            .unwrap_or(&content_type::UNKNOWN);

        parent.unwrap().to(file_name.to_string(), ext)
    }
}

impl Clone for DirNode {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
        }
    }
}

impl Clone for FileNode {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
            content_type: self.content_type,
            file: self.file.clone(),
        }
    }
}

impl DirNode {
    pub fn mkdir(&self) -> Res<()> {
        fs::create_dir_all(self.absolute_path()).map_err(warp_e)
    }

    pub fn next(&self, name: String) -> DirNode {
        DirNode {
            node: Arc::new(RwLock::new((name, Some(self.clone())))),
        }
    }

    pub fn to(&self, name: String, context_type: &'static ContentType) -> FileNode {
        FileNode {
            node: Arc::new(RwLock::new((name, Some(self.clone())))),
            content_type: context_type,
            file: Arc::new(RefCell::new(None)),
        }
    }
}

impl FileNode {
    pub fn touch(&self) -> Res<()> {
        self.up().unwrap().mkdir()?;
        self.file.replace(Option::from(
            File::create(self.absolute_path()).map_err(warp_e)?,
        ));
        Ok(())
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Res<usize> {
        self.open_file(false)?;
        match self.file.try_borrow_mut()?.as_mut() {
            None => Err(warp("File not exists")),
            Some(file) => file.read(buf).map_err(warp_e),
        }
    }

    pub fn read_left(&mut self, buf: &mut Vec<u8>) -> Res<usize> {
        self.open_file(false)?;
        self.read_to_end(buf)
    }

    pub fn read_all(&mut self, buf: &mut Vec<u8>) -> Res<usize> {
        self.open_file(true)?;
        self.read_to_end(buf)
    }

    pub fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Res<usize> {
        match self.file.try_borrow_mut()?.as_mut() {
            None => Err(warp("File not exists")),
            Some(file) => file.read_to_end(buf).map_err(warp_e),
        }
    }

    pub fn read_json<T>(&mut self) -> Res<T>
    where
        T: de::DeserializeOwned,
    {
        self.open_file(true)?;
        match self.file.try_borrow_mut()?.as_mut() {
            None => Err(warp("File not exists")),
            Some(file) => {
                let mut buf = String::new();
                file.read_to_string(&mut buf).map_err(warp_e)?;
                json::parse(&buf)
            }
        }
    }

    pub fn as_stream(&self) -> Res<FileStream> {
        self.open_file(true)?;
        let file = self.file.replace(None);
        if let Some(file) = file {
            Ok(FileStream {
                file: RefCell::new(file),
            })
        } else {
            Err(NoSuchFile)
        }
    }

    pub fn write(&mut self, buf: &[u8]) -> Res<usize> {
        self.touch()?;
        match self.file.try_borrow_mut()?.as_mut() {
            None => Err(warp("File not exists")),
            Some(file) => file.write(buf).map_err(warp_e),
        }
    }

    pub fn set_content_type(&mut self, content_type: &'static ContentType) {
        self.content_type = content_type;
    }

    fn open_file(&self, reopen: bool) -> Res<()> {
        if self.file.borrow().is_none() || reopen {
            self.file
                .replace(Some(File::open(self.absolute_path()).map_err(warp_e)?));
        }
        Ok(())
    }
}

impl DirNode {
    static_root_volume!(A, "A:");
    static_root_volume!(B, "B:");
    static_root_volume!(C, "C:");
    static_root_volume!(D, "D:");
    static_root_volume!(E, "E:");
    static_root_volume!(F, "F:");
    static_root_volume!(G, "G:");
    static_root_volume!(H, "H:");
    static_root_volume!(I, "I:");
    static_root_volume!(J, "J:");
    static_root_volume!(K, "K:");
    static_root_volume!(L, "L:");
    static_root_volume!(M, "M:");
    static_root_volume!(N, "N:");
    static_root_volume!(O, "O:");
    static_root_volume!(P, "P:");
    static_root_volume!(Q, "Q:");
    static_root_volume!(R, "R:");
    static_root_volume!(S, "S:");
    static_root_volume!(T, "T:");
    static_root_volume!(U, "U:");
    static_root_volume!(V, "V:");
    static_root_volume!(W, "W:");
    static_root_volume!(X, "X:");
    static_root_volume!(Y, "Y:");
    static_root_volume!(Z, "Z:");

    static_root_volume!(ROOT, "");
    static_root_volume!(HOME, "~");

    pub fn new(name: String) -> Self {
        Self {
            node: Arc::new(RwLock::new((name, None))),
        }
    }
}

pub mod content_type {
    use crate::common::{Error, Res};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub enum FileType {
        Unknown,
        Plain,
        Image,
    }

    pub struct ContentType {
        pub id: i64,
        pub file: FileType,
        pub ext: &'static str,
        pub mimetype: &'static str,
        pub magic: &'static [u8],
    }

    static_file_context_type!(0x000, UNKNOWN, FileType::Unknown, "", "application/octet-stream", &[]);
    static_file_context_type!(0x001, JSON, FileType::Plain, ".json", "application/json", &[]);
    static_file_context_type!(0x002, GIF87A, FileType::Image, ".gif", "image/gif", &[0x47, 0x49, 0x46, 0x38, 0x37, 0x61]);
    static_file_context_type!(0x003, GIF89A, FileType::Image, ".gif", "image/gif", &[0x47, 0x49, 0x46, 0x38, 0x39, 0x61]);
    static_file_context_type!(0x004, BMP, FileType::Image, ".bmp", "image/bmp", &[0x42, 0x4D]);
    static_file_context_type!(0x005, JPEG, FileType::Image, ".jpg", "image/jpeg", &[0xFF, 0xD8, 0xFF]);
    static_file_context_type!(0x006, PNG, FileType::Image, ".png", "image/png", &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    static_file_context_type!(0x007, HEIF, FileType::Image, ".heif", "image/heif", &[0x66, 0x74, 0x79, 0x70]);
    static_file_context_type!(0x008, TIFF1, FileType::Image, ".tif", "image/tiff", &[0x49, 0x49, 0x2A, 0x00]);
    static_file_context_type!(0x009, TIFF2, FileType::Image, ".tiff", "image/tiff", &[0x4D, 0x4D, 0x00, 0x2A]);

    const ALL: &'static [ContentType] = &[
        UNKNOWN, JSON, GIF87A, GIF89A, BMP, JPEG, PNG, HEIF, TIFF1, TIFF2,
    ];

    pub fn file_check(data: &Vec<u8>) -> Res<&'static ContentType> {
        if data.len() < 8 {
            return Err(Error::UnknownFileContentType);
        }

        for content_type in ALL.iter() {
            if content_type.magic.len() > 0
                && data
                    .iter()
                    .zip(content_type.magic.iter())
                    .all(|(d, m)| d == m)
            {
                return Ok(content_type);
            }
        }

        Err(Error::UnknownFileContentType)
    }

    pub fn from(id: i64) -> &'static ContentType {
        ALL.iter()
            .find(|content_type| content_type.id == id)
            .unwrap_or(&UNKNOWN)
    }

    pub fn guess(ext: &str) -> &'static ContentType {
        ALL.iter()
            .find(|content_type| content_type.ext == ext)
            .unwrap_or(&UNKNOWN)
    }

    impl Serialize for ContentType {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_i64(self.id)
        }
    }

    impl<'de> Deserialize<'de> for &'static ContentType {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            Ok(from(i64::deserialize(deserializer)?))
        }
    }
}

fn warp(s: &str) -> Error {
    Error::DirectoryError(String::from(s))
}

fn warp_e(e: std::io::Error) -> Error {
    Error::DirectoryError(e.to_string())
}

pub struct FileStream {
    file: RefCell<File>,
}

impl Stream for FileStream {
    type Item = Res<Bytes>;

    fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut buffer = [0; 1024];
        let res = self.file.borrow_mut().read(&mut buffer);
        if let Ok(size) = res {
            if size == 0 {
                Ready(None)
            } else {
                let mut vec = Vec::from(buffer);
                vec.truncate(size);
                let bytes = Bytes::from(vec);
                Ready(Some(Ok(bytes)))
            }
        } else {
            Ready(Some(Err(NoSuchFile)))
        }
    }
}
