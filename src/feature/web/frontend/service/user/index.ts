import {request} from "@/service";

const check = (callback: () => void) => {
  request({
    method: "GET",
    url: "/api/user/check",
    success: callback
  })
}

export {
  check
}