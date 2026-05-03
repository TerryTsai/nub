import { lazy, Suspense } from "react";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { Layout } from "./components/Layout";
import { ScrollRestoration } from "./components/ScrollRestoration";
import { Toaster } from "./components/Toaster";
import { Hosts } from "./screens/Hosts";
import { AddHost } from "./screens/AddHost";
import { HostHome } from "./screens/HostHome";
import { ContainerDetail } from "./screens/ContainerDetail";
import { ContainerLogs } from "./screens/ContainerLogs";
import { ContainerStats } from "./screens/ContainerStats";
import { DockerfileEdit } from "./screens/DockerfileEdit";
import { HostDockerfiles } from "./screens/HostDockerfiles";
import { HostImages } from "./screens/HostImages";
import { HostNetworks } from "./screens/HostNetworks";
import { HostVolumes } from "./screens/HostVolumes";
import { ImageDetail } from "./screens/ImageDetail";
import { NetworkDetail } from "./screens/NetworkDetail";
import { NewContainer } from "./screens/NewContainer";
import { NewImage } from "./screens/NewImage";
import { VolumeDetail } from "./screens/VolumeDetail";

// Exec carries xterm.js (~340KB). Lazy-load so it doesn't bloat the
// initial bundle for users who never open a terminal.
const ContainerExec = lazy(() =>
  import("./screens/ContainerExec").then((m) => ({ default: m.ContainerExec })),
);

export default function App() {
  return (
    <BrowserRouter>
      <ScrollRestoration />
      <Toaster>
      <Routes>
        <Route element={<Layout />}>
          <Route path="/" element={<Hosts />} />
          <Route path="/add" element={<AddHost />} />
          <Route path="/h/:hid" element={<HostHome />} />
          <Route path="/h/:hid/images" element={<HostImages />} />
          <Route path="/h/:hid/images/new" element={<NewImage />} />
          <Route path="/h/:hid/images/:iid" element={<ImageDetail />} />
          <Route path="/h/:hid/volumes" element={<HostVolumes />} />
          <Route path="/h/:hid/volumes/:vname" element={<VolumeDetail />} />
          <Route path="/h/:hid/networks" element={<HostNetworks />} />
          <Route path="/h/:hid/networks/:nid" element={<NetworkDetail />} />
          <Route path="/h/:hid/dockerfiles" element={<HostDockerfiles />} />
          <Route path="/h/:hid/dockerfiles/:name" element={<DockerfileEdit />} />
          <Route path="/h/:hid/c/new" element={<NewContainer />} />
          <Route path="/h/:hid/c/:cid/clone" element={<NewContainer />} />
          <Route path="/h/:hid/c/:cid" element={<ContainerDetail />} />
          <Route path="/h/:hid/c/:cid/logs" element={<ContainerLogs />} />
          <Route path="/h/:hid/c/:cid/stats" element={<ContainerStats />} />
          <Route
            path="/h/:hid/c/:cid/exec"
            element={
              <Suspense fallback={<p className="px-5 pt-5 text-xs text-[var(--text-tertiary)]">Loading terminal…</p>}>
                <ContainerExec />
              </Suspense>
            }
          />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Route>
      </Routes>
      </Toaster>
    </BrowserRouter>
  );
}
