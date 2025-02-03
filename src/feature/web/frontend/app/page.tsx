"use client"

import React, {useState} from "react";
import Input from "@/components/input";
import Button from "@/components/button";
// noinspection JSUnusedGlobalSymbols
export default function Home() {

  const [v, setV] = useState(false)

  return (
    <div style={{display: "grid", placeItems: "center", height: "100vh"}}>
      <Input label="test" type="NUMBER" front="width:" back="px"></Input>
      <div style={{display: "grid", gridTemplateColumns: "1fr 1fr 1fr 1fr 1fr"}}>
        <Button label="Default Button" loading={v} onClick={() => setV(!v)} type="primary" level="BRAND"></Button>
        <Button label="Default Button" loading={v} onClick={() => setV(!v)} type="default" level="BRAND"></Button>
        <Button label="Default Button" loading={v} onClick={() => setV(!v)} type="dashed" level="BRAND"></Button>
        <Button label="Default Button" loading={v} onClick={() => setV(!v)} type="text" level="BRAND"></Button>
        <Button label="Default Button" loading={v} onClick={() => setV(!v)} type="link" level="BRAND"></Button>

        <Button label="Default Button" loading={v} onClick={() => setV(!v)} type="primary" level="SUCCESS"></Button>
        <Button label="Default Button" loading={v} onClick={() => setV(!v)} type="default" level="SUCCESS"></Button>
        <Button label="Default Button" loading={v} onClick={() => setV(!v)} type="dashed" level="SUCCESS"></Button>
        <Button label="Default Button" loading={v} onClick={() => setV(!v)} type="text" level="SUCCESS"></Button>
        <Button label="Default Button" loading={v} onClick={() => setV(!v)} type="link" level="SUCCESS"></Button>

        <Button label="Default Button" loading={v} onClick={() => setV(!v)} type="primary" level="INFO"></Button>
        <Button label="Default Button" loading={v} onClick={() => setV(!v)} type="default" level="INFO"></Button>
        <Button label="Default Button" loading={v} onClick={() => setV(!v)} type="dashed" level="INFO"></Button>
        <Button label="Default Button" loading={v} onClick={() => setV(!v)} type="text" level="INFO"></Button>
        <Button label="Default Button" loading={v} onClick={() => setV(!v)} type="link" level="INFO"></Button>

        <Button label="Default Button" loading={v} onClick={() => setV(!v)} type="primary" level="WARNING"></Button>
        <Button label="Default Button" loading={v} onClick={() => setV(!v)} type="default" level="WARNING"></Button>
        <Button label="Default Button" loading={v} onClick={() => setV(!v)} type="dashed" level="WARNING"></Button>
        <Button label="Default Button" loading={v} onClick={() => setV(!v)} type="text" level="WARNING"></Button>
        <Button label="Default Button" loading={v} onClick={() => setV(!v)} type="link" level="WARNING"></Button>

        <Button label="Default Button" loading={v} onClick={() => setV(!v)} type="primary" level="ERROR"></Button>
        <Button label="Default Button" loading={v} onClick={() => setV(!v)} type="default" level="ERROR"></Button>
        <Button label="Default Button" loading={v} onClick={() => setV(!v)} type="dashed" level="ERROR"></Button>
        <Button label="Default Button" loading={v} onClick={() => setV(!v)} type="text" level="ERROR"></Button>
        <Button label="Default Button" loading={v} onClick={() => setV(!v)} type="link" level="ERROR"></Button>
      </div>
    </div>
  )
}
