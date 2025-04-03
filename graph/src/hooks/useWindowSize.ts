"use client"; // ensure it's a client component
import { useState, useEffect } from "react";

export function useWindowSize() {
  const isClient = typeof window !== "undefined";
  const [size, setSize] = useState<{ width: number; height: number }>(
    isClient ? { width: window.innerWidth, height: window.innerHeight } : { width: 0, height: 0 }
  );

  useEffect(() => {
    if (!isClient) return;
    function handleResize() {
      setSize({ width: window.innerWidth, height: window.innerHeight });
    }
    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, [isClient]);

  return size;
}