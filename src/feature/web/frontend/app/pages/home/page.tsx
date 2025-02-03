"use client"

import Tag from "@/components/tag";

export default function Home() {
  return (
    <div style={{margin: '10px'}}>
      <Tag name="test" type="SUCCESS" closeable={true} close={() => console.log(1)}></Tag>
      <Tag name="test" type="WARNING"></Tag>
      <Tag name="test" type="ERROR"></Tag>
      <Tag name="test" type="INFO"></Tag>
    </div>
  )
}