//! Engine wire shapes for `POST /containers/{id}/exec` and the
//! follow-up `POST /exec/{id}/start` upgrade body.

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub(in crate::ops::containers) struct CreateExecBody {
    pub attach_stdin: bool,
    pub attach_stdout: bool,
    pub attach_stderr: bool,
    pub tty: bool,
    pub cmd: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(in crate::ops::containers) struct CreateExecResp {
    #[serde(rename = "Id")]
    pub id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub(in crate::ops::containers) struct StartExecBody {
    pub detach: bool,
    pub tty: bool,
}
