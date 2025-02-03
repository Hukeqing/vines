import "./globals.css"
import "./iconfont.css"

import {MessageContainer} from "@/components/message";
import React from "react";

// noinspection JSUnusedGlobalSymbols
export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body>
        {children}
        <MessageContainer />
      </body>
    </html>
  )
}
