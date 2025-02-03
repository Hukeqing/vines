"use client"

import "./style.css"
import React, {useEffect, useRef, useState} from 'react'

interface Index {
  type: "INFO" | "SUCCESS" | "WARNING" | "ERROR",
  title: string,
  body: string | React.ReactNode,
  duration: "auto" | number
}

interface InnerMessage {
  id: number,
  type: "INFO" | "SUCCESS" | "WARNING" | "ERROR",
  title: string,
  body: string | React.ReactNode,
  duration: number
}

function MessageBox({id, type, title, body, duration}: InnerMessage) {
  const [holderHeight, setHolderHeight] = useState(0)
  const [status, setStatus] = useState<'NEW'|'POP'|'STAY'|'PIN'|'CLOSE'>('NEW')
  const ref = useRef<HTMLDivElement>(null)
  const refStatus = useRef<'NEW'|'POP'|'STAY'|'PIN'|'CLOSE'>('NEW')
  const preHeight = useRef(0)

  const timeout = useRef<ReturnType<typeof setTimeout> | null>(null)
  const fresher = useRef<ReturnType<typeof setInterval> | null>(null)

  const checkThen = (need: 'NEW'|'POP'|'STAY'|'PIN'|'CLOSE', target: 'NEW'|'POP'|'STAY'|'PIN'|'CLOSE') => {
    if (refStatus.current !== need) return false
    setStatus(target)
    refStatus.current = target
    return true
  }

  const toPop = () => {
    if (!checkThen('NEW', 'POP')) return
    fresher.current = setInterval(fresh, 10)
  }

  const toStay = () => {
    if (!checkThen('POP', 'STAY')) return
    timeout.current = setTimeout(toClose, duration)
    if (fresher.current) clearInterval(fresher.current!)
    fresher.current = null
  }

  const toPin = () => {
    if (!checkThen('STAY', 'PIN')) return
    if (timeout.current) clearTimeout(timeout.current!)
    timeout.current = null
  }

  const toClose = () => {
    if (!checkThen('STAY', 'CLOSE') && !checkThen('PIN', 'CLOSE')) return
    timeout.current = setTimeout(end, 500)
    fresher.current = setInterval(fresh, 10)
  }

  const end = () => {
    if (timeout.current) clearTimeout(timeout.current!);
    if (fresher.current) clearInterval(fresher.current!);
    pop_message(id);
  }

  useEffect(toPop, [id])

  const fresh = () => {
    const newHeight = ref.current?.getBoundingClientRect().height ?? 0;

    setHolderHeight(newHeight)
    if (refStatus.current === 'POP' && newHeight > 0 && preHeight.current === newHeight) {
      toStay()
    }
    if (refStatus.current === 'CLOSE' && newHeight === 0) {
      end()
    }

    preHeight.current = newHeight
  }

  const messageStyle = {
    animation: `0.3s ease ${status !== 'CLOSE' ? 'message-box-show forwards': 'message-box-close forwards'}`,
    backgroundColor: `var(--light-${type.toLowerCase()}-color)`,
    color: `var(--${type.toLowerCase()}-color)`,
  }

  const messageHolderStyle = {
    height: `${holderHeight}px`,
    animation: `0.3s ease ${status !== 'CLOSE' ? 'message-holder-show forwards': 'message-holder-close forwards'}`
  }

  return (
    <div>
      <div className="message-box" style={messageStyle} ref={ref}>
        <div className={`message-icon ${status === 'STAY' ? 'message-icon-enable' : ''}`} onClick={toPin}>
          {status !== 'PIN' && <i className="iconfont icon-pin"/>}
        </div>
        <div>
          <div className="message-title">{title} {id}</div>
          <div className="message-body">{body}</div>
        </div>
        <div className={`message-icon ${status === 'STAY' || status === 'PIN' ? 'message-icon-enable' : ''}`}
             onClick={toClose}>
          <i className="iconfont icon-close"/>
        </div>
      </div>
      <div className="message-holder" style={messageHolderStyle}></div>
    </div>
  )
}

let pop_message: OmitThisParameter<(index: number) => void>
let message: OmitThisParameter<(msg: Index) => void>

function MessageContainer() {
  const [messageList, setMessageList] = useState<InnerMessage[]>([])
  const index = useRef(0)

  message = (msg: Index) => {
    setMessageList([{
      id: index.current++,
      type: msg.type,
      title: msg.title,
      body: msg.body,
      duration: msg.duration === 'auto' ? (typeof msg.body === 'string' ? msg.body.length * 500 : 10000) : msg.duration,
    }, ...messageList])
  }

  pop_message = (index: number) => {
    setMessageList(messageList.filter(v => v.id != index))
  }

  return (
    <div className="message-container">
      {messageList.map(msg => (
        <MessageBox key={msg.id} id={msg.id} type={msg.type} title={msg.title} body={msg.body} duration={msg.duration}/>
      ))}
    </div>
  )
}

export {
  MessageContainer,
  message
}
