import React from "react";

type KeyType = string | number | null | undefined;
type LevelType = "BRAND" | "SUCCESS" | "INFO" | "WARNING" | "ERROR";

const render = (text?: unknown) => {
  if (text === null || text === undefined) {
    return <div>Empty</div>;
  }

  if (typeof text === "string" || typeof text === "number" || typeof text === "boolean") {
    return <div>{String(text)}</div>;
  }

  if (React.isValidElement(text)) {
    return text;
  }

  if (Array.isArray(text)) {
    return <div>{text.map((item, index) => <div key={index}>{render(item)}</div>)}</div>;
  }

  if (typeof text === "object") {
    try {
      return <pre>{JSON.stringify(text, null, 2)}</pre>;
    } catch {
      return <div>Invalid object</div>;
    }
  }

  return <div>Unsupported type</div>;
};

export {
  render
};

export type {
  KeyType,
  LevelType
};
