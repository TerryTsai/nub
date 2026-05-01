import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { Hosts } from "./screens/Hosts";
import { AddHost } from "./screens/AddHost";
import { HostHome } from "./screens/HostHome";
import { ContainerDetail } from "./screens/ContainerDetail";

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Hosts />} />
        <Route path="/add" element={<AddHost />} />
        <Route path="/h/:hid" element={<HostHome />} />
        <Route path="/h/:hid/c/:cid" element={<ContainerDetail />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </BrowserRouter>
  );
}
