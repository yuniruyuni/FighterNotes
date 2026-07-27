import { useEffect, useState } from "react";

export function useObjectUrl(value: Blob | null): string {
  const [url, setUrl] = useState("");

  useEffect(() => {
    if (!value) {
      setUrl("");
      return;
    }
    const nextUrl = URL.createObjectURL(value);
    setUrl(nextUrl);
    return () => URL.revokeObjectURL(nextUrl);
  }, [value]);

  return url;
}
