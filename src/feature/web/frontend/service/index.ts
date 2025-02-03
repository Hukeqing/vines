import axios from "axios";

interface RequestParam {
  method: 'GET' | 'POST'
  url: string
  query?: object
  body?: object
  success?: (v: object) => void
  error?: (v: object) => void
  final?: () => void
}

const request = ({method, url, query, body, success, error, final}: RequestParam) => {
  axios({
    method,
    url,
    params: query,
    data: body
  }).then(data => {
    if (success) success(data)
    if (final) final()
  }).catch(err => {
    if (error) error(err)
    if (final) final()
  })
}

export {
  request
}