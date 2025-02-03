import {FormEvent, FocusEvent, useState} from "react";
import {render} from "@/common/common";
import "./style.css"
import cn from "classnames";

type InputType = "STRING" | "NUMBER" | "PASSWORD";

interface InputParam {
  label: string
  type: InputType
  holder?: string
  front?: string
  back?: string
  before?: (v: string) => string
  change?: (v: string) => void
}

export default function Input({label, holder = "", front, back, type, before, change}: InputParam) {

  const [value, setValue] = useState(holder);
  const [focus, setFocus] = useState(false)

  const beforeChange = (newValue: string) => {
    switch (type) {
      case "STRING":
      case "PASSWORD":
        return newValue;
      case "NUMBER":
        const num = Number(newValue);
        if (!Number.isNaN(num)) return newValue;
        break;
    }

    return value;
  }

  const afterChange = (newValue: string) => {
    if (change) change(newValue)
  }

  const onInput = (event: FormEvent) => {
    const oldValue = (event.target as HTMLInputElement).value;
    // before
    let newValue = beforeChange(oldValue)
    if (before) newValue = before(newValue)
    // after
    setValue(newValue)
    afterChange(newValue)
  }

  const onFocus = (event: FocusEvent) => {
    setFocus(true)
  }

  const onBlur = (event: FocusEvent) => {
    setFocus(false)
  }

  const renderInput = (type: InputType) => {
    switch (type) {
      case "STRING":
      case "NUMBER":
        return <input value={value}
                      onInput={onInput}
                      className="input-dom"
                      onFocus={onFocus}
                      onBlur={onBlur}/>
      case "PASSWORD":
        return <input value={value}
                      type="password"
                      onInput={onInput}
                      className="input-dom"
                      onFocus={onFocus}
                      onBlur={onBlur}/>
    }
  }

  return (
    <div className="input-container">
      <div className="input-box">
        <div>{front}</div>
        <div className="input-with-label">
          {renderInput(type)}
          <div className={cn("input-label", {
            "input-label-without-word": value === "",
            "input-label-with-word": value !== "",
          })}>{render(label)}</div>
        </div>
        <div>{back}</div>
      </div>

      <div className={cn("input-underline", {
        "input-underline-focus": focus
      })}></div>
    </div>
  )
}