import {request} from "@/service";

const list = (callback: (v: object) => void) => {
  request({
    method: "GET",
    url: "/api/repo/list",
    success: callback
  })
}

export {
  list
}
