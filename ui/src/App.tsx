import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { Hosts } from "./screens/Hosts";
import { AddHost } from "./screens/AddHost";
import { HostHome } from "./screens/HostHome";
import { ContainerDetail } from "./screens/ContainerDetail";
import { ContainerLogs } from "./screens/ContainerLogs";
import { ContainerStats } from "./screens/ContainerStats";
import { RunContainer } from "./screens/RunContainer";

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Hosts />} />
        <Route path="/add" element={<AddHost />} />
        <Route path="/h/:hid" element={<HostHome />} />
        <Route path="/h/:hid/run" element={<RunContainer />} />
        <Route path="/h/:hid/c/:cid" element={<ContainerDetail />} />
        <Route path="/h/:hid/c/:cid/logs" element={<ContainerLogs />} />
        <Route path="/h/:hid/c/:cid/stats" element={<ContainerStats />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </BrowserRouter>
  );
}
