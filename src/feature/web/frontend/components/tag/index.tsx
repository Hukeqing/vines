import "./style.css"

interface TagParam {
  name: string,
  type: "INFO" | "SUCCESS" | "WARNING" | "ERROR",
  closeable?: boolean
  close?: () => void,
}

export default function Tag({name, type, closeable, close}: TagParam) {

  // region css

  const tagStyle = {
    backgroundColor: `var(--light-${type.toLowerCase()}-color)`,
    color: `var(--${type.toLowerCase()}-color)`,
    border: `1px solid var(--${type.toLowerCase()}-color)`
  }

  // endregion

  return (
    <div className="tag-container" style={tagStyle}>
      {name}
      {(closeable ?? false) ?
        <span className="tag-close" onClick={() => close && close()}>
          <i className="iconfont icon-close"></i>
        </span> :
        <span />}
    </div>
  )
}
