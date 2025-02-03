"use client"

import "./style.css"

import React from "react";
import Navigation from "@/components/navigation";

// noinspection JSUnusedGlobalSymbols
export default function RootLayout({children,}: {
  children: React.ReactNode
}) {
  const items = [
    {
      value: 10,
      label: "test2",
      icon: "icon-pin",
    }, {
      value: 20,
      label: "test",
      icon: "icon-pin",
    }, {
      value: 30,
      label: "test3",
      icon: "icon-pin",
    },
  ]

  return (
    <div style={{display: 'grid', gridTemplateColumns: "auto 1fr", gap: "10px"}}>
      <Navigation def={10} items={items} collapse={true} collapseLabel="收起" change={v => console.log(v)}/>
      {children}
    </div>
  )
}
