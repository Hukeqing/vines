import React, {useEffect, useState} from "react";
import "./style.css"
import {render} from "@/common/common"
import {useRouter} from 'next/navigation'
import {KeyType} from "@/common/common";

interface NavParam {
  def: KeyType,
  change?: (v: KeyType) => void,
  collapse?: boolean,
  collapseLabel?: string | React.ReactNode,
  items: NavItem[],
}

interface NavItem {
  value: KeyType,
  icon?: string | React.ReactNode,
  label?: string,
  hidden?: boolean,
  link?: string
}

export default function Navigation({def, change, collapse, collapseLabel, items}: NavParam) {

  const router = useRouter()

  const [collapsed, setCollapsed] = useState(false)
  const [current, setCurrent] = useState(def)

  const containerStyle = {
    width: (collapsed ?? false) ? '50px' : '200px'
  }

  // region function

  const update = (v: KeyType, link?: string) => {
    setCurrent(v)
    if (link) router.push(link)
    if (change) change(v)
  }

  useEffect(() => {
    setCollapsed(collapse ?? false)
  }, [collapse])

  // endregion

  // region css

  const collapseStyle = {
    transition: '0.3s ease-in-out transform',
    transform: `rotate(${collapsed ? 180 : 0}deg)`
  }

  // endregion

  // region render

  const renderIcon = (icon?: string | React.ReactNode) => {
    if (!icon) {
      return (<div></div>)
    } else if (typeof icon === 'string') {
      return (<div className="navigation-icon"><i className={`iconfont ${icon}`}/></div>)
    } else {
      return icon
    }
  }

  const renderCollapse = () => (
    <div className="navigation-menu navigation-menu-unselected" onClick={() => setCollapsed(!collapsed)}>
      <div style={collapseStyle} className="navigation-icon"><i className="iconfont icon-collapse"/></div>
      {render(collapseLabel)}
    </div>
  )

  const renderRow = ({value, icon, label, hidden, link}: NavItem) => (
    hidden ? <div key={value} className="navigation-menu-hidden"></div> :
      <div key={value}
           className={`navigation-menu ${value === current ? 'navigation-menu-selected' : 'navigation-menu-unselected'}`}
           onClick={() => update(value, link)}>
        {renderIcon(icon)}
        {collapsed ? <div/> : <div>{label ?? render(value)}</div>}
      </div>
  )

  const renderList = (list: NavItem[]) => (<div>{list.map(renderRow)}</div>)

  // endregion

  return (
    <div className="navigation-container" style={containerStyle}>
      {collapseLabel && renderCollapse()}
      {renderList(items)}
    </div>
  )
}
