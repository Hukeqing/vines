import './style.css'
import {LevelType} from "@/common/common";
import cn from "classnames";
import React, {useState} from "react";

type TypeEnum = "primary" | "default" | "dashed" | "text" | "link";

interface ButtonParam {
  children?: React.ReactNode;
  type?: TypeEnum
  level?: LevelType
  loading?: boolean
  disable?: boolean
  onClick?: () => void
}

export default function Button({
                                 children,
                                 type = "default",
                                 level = "BRAND",
                                 loading = false,
                                 disable = false,
                                 onClick
                               }: ButtonParam) {
  const [animation, setAnimation] = useState("")

  const mouseDown = () => {
    if (disable || loading) {
      return
    }
    setAnimation("")
    setTimeout(() => {
      setAnimation("shadow-down")
    }, 0)
  }

  const mouseUp = () => {
    if (disable || loading) {
      return
    }
    setAnimation("shadow-down")
    setTimeout(() => {
      setAnimation("shadow-up")
    }, 0)
    if (onClick) onClick()
  }

  const typeClassNameBuild = () => {
    if (disable) {
      switch (type) {
        case "default":
        case "primary":
          return 'fill-disable'
        case "dashed":
          return 'dashed-disable'
        case "text":
        case "link":
          return 'text-disable'
      }
    }

    return type
  }

  const renderIcon = () => {
    return loading
      ? (<div className="icon-circle"><i className="iconfont icon-loading"></i></div>)
      : (<div></div>)
  }

  return (
    <div onMouseDown={mouseDown}
         onMouseUp={mouseUp}
         className={cn(
           "button-container",
           `level-${level?.toLowerCase()}-color`,
           `button-container-${typeClassNameBuild()}`)}>
      <div className="button-shadow" style={{
        animation: `${animation} 200ms ease-in forwards`
      }}></div>
      <button className="button">
        {renderIcon()}
        {children}
      </button>
    </div>
  )
}