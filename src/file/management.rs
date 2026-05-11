/// File management module
///
/// Provides file and folder management functionality (create, rename, delete, move, copy)
use log::{debug, info};
use serde::Deserialize;

use super::FileClient;
use crate::auth::AccessToken;
use crate::errors::{NetDiskError, NetDiskResult};

/// Extension trait for file management operations
pub(crate) trait FileManagementExt {
    fn create_folder(
        &self,
        access_token: &AccessToken,
        path: &str,
    ) -> impl std::future::Future<Output = NetDiskResult<FolderInfo>> + Send;

    fn create_folder_with_options(
        &self,
        access_token: &AccessToken,
        path: &str,
        options: FolderCreateOptions,
    ) -> impl std::future::Future<Output = NetDiskResult<FolderInfo>> + Send;

    fn rename(
        &self,
        access_token: &AccessToken,
        path: &str,
        new_name: &str,
    ) -> impl std::future::Future<Output = NetDiskResult<()>> + Send;

    fn delete(
        &self,
        access_token: &AccessToken,
        path: &str,
    ) -> impl std::future::Future<Output = NetDiskResult<()>> + Send;

    fn move_file(
        &self,
        access_token: &AccessToken,
        path: &str,
        dest: &str,
    ) -> impl std::future::Future<Output = NetDiskResult<()>> + Send;

    fn copy_file(
        &self,
        access_token: &AccessToken,
        path: &str,
        dest: &str,
    ) -> impl std::future::Future<Output = NetDiskResult<()>> + Send;
}

impl FileManagementExt for FileClient {
    async fn create_folder(
        &self,
        access_token: &AccessToken,
        path: &str,
    ) -> NetDiskResult<FolderInfo> {
        self.create_folder_with_options(access_token, path, FolderCreateOptions::default())
            .await
    }

    async fn create_folder_with_options(
        &self,
        access_token: &AccessToken,
        path: &str,
        options: FolderCreateOptions,
    ) -> NetDiskResult<FolderInfo> {
        let params = [
            ("method", "create"),
            ("access_token", &access_token.access_token),
        ];

        let form = [
            ("path", path),
            ("isdir", "1"),
            ("rtype", &options.rtype.to_string()),
            ("mode", &options.mode.to_string()),
        ];

        debug!(
            "Creating folder at path: {} with options: {:?}",
            path, options
        );

        let response: FolderResponse = self
            .http_client()
            .post_form("/rest/2.0/xpan/file", Some(&form), Some(&params))
            .await?;

        if response.errno != 0 {
            let errmsg = response.errmsg.as_deref().unwrap_or("Unknown error");
            return Err(NetDiskError::api_error(response.errno, errmsg));
        }

        info!("Folder created successfully at: {}", path);

        Ok(FolderInfo {
            fs_id: response.fs_id,
            path: response.path.unwrap_or_else(|| path.to_string()),
            ctime: response.ctime,
            mtime: response.mtime,
            isdir: response.isdir,
            category: response.category,
        })
    }

    async fn rename(
        &self,
        access_token: &AccessToken,
        path: &str,
        new_name: &str,
    ) -> NetDiskResult<()> {
        let filelist = serde_json::json!([
            {
                "path": path,
                "newname": new_name
            }
        ]);

        let params = [
            ("method", "filemanager"),
            ("access_token", &access_token.access_token),
            ("opera", "rename"),
        ];

        let form = [
            ("async", "0"),
            ("filelist", &filelist.to_string()),
            ("ondup", "overwrite"),
        ];

        debug!("Renaming path: {} to: {}", path, new_name);

        let response: FileOperationResponse = self
            .http_client()
            .post_form("/rest/2.0/xpan/file", Some(&form), Some(&params))
            .await?;

        if response.errno != 0 {
            return Err(NetDiskError::api_error(response.errno, "Rename failed"));
        }

        info!("Renamed successfully: {} -> {}", path, new_name);
        Ok(())
    }

    async fn delete(&self, access_token: &AccessToken, path: &str) -> NetDiskResult<()> {
        let filelist = serde_json::json!([
            {
                "path": path
            }
        ]);

        let params = [
            ("method", "filemanager"),
            ("access_token", &access_token.access_token),
            ("opera", "delete"),
        ];

        let form = [
            ("async", "0"),
            ("filelist", &filelist.to_string()),
            ("ondup", "overwrite"),
        ];

        debug!("Deleting path: {}", path);

        let response: FileOperationResponse = self
            .http_client()
            .post_form("/rest/2.0/xpan/file", Some(&form), Some(&params))
            .await?;

        if response.errno != 0 {
            return Err(NetDiskError::api_error(response.errno, "Delete failed"));
        }

        info!("Deleted successfully: {}", path);
        Ok(())
    }

    async fn move_file(
        &self,
        access_token: &AccessToken,
        path: &str,
        dest: &str,
    ) -> NetDiskResult<()> {
        let filename = path.rsplit('/').next().unwrap_or(path);

        let filelist = serde_json::json!([
            {
                "path": path,
                "dest": dest,
                "newname": filename,
                "ondup": "fail"
            }
        ]);

        let params = [
            ("method", "filemanager"),
            ("access_token", &access_token.access_token),
            ("opera", "move"),
        ];

        let form = [
            ("async", "0"),
            ("filelist", &filelist.to_string()),
            ("ondup", "fail"),
        ];

        debug!("Moving path: {} to: {}", path, dest);

        let response: FileOperationResponse = self
            .http_client()
            .post_form("/rest/2.0/xpan/file", Some(&form), Some(&params))
            .await?;

        if response.errno != 0 {
            return Err(NetDiskError::api_error(response.errno, "Move failed"));
        }

        info!("Moved successfully: {} -> {}", path, dest);
        Ok(())
    }

    async fn copy_file(
        &self,
        access_token: &AccessToken,
        path: &str,
        dest: &str,
    ) -> NetDiskResult<()> {
        let filename = path.rsplit('/').next().unwrap_or(path);

        let filelist = serde_json::json!([
            {
                "path": path,
                "dest": dest,
                "newname": filename,
                "ondup": "fail"
            }
        ]);

        let params = [
            ("method", "filemanager"),
            ("access_token", &access_token.access_token),
            ("opera", "copy"),
        ];

        let form = [
            ("async", "0"),
            ("filelist", &filelist.to_string()),
            ("ondup", "fail"),
        ];

        debug!("Copying path: {} to: {}", path, dest);

        let response: FileOperationResponse = self
            .http_client()
            .post_form("/rest/2.0/xpan/file", Some(&form), Some(&params))
            .await?;

        if response.errno != 0 {
            return Err(NetDiskError::api_error(response.errno, "Copy failed"));
        }

        info!("Copied successfully: {} -> {}", path, dest);
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct FolderCreateOptions {
    pub rtype: i32,
    pub mode: i32,
}

impl FolderCreateOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rtype(mut self, rtype: i32) -> Self {
        self.rtype = rtype;
        self
    }

    pub fn mode(mut self, mode: i32) -> Self {
        self.mode = mode;
        self
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct FolderInfo {
    pub fs_id: Option<u64>,
    pub path: String,
    pub ctime: Option<u64>,
    pub mtime: Option<u64>,
    pub isdir: Option<i32>,
    pub category: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct FolderResponse {
    errno: i32,
    errmsg: Option<String>,
    fs_id: Option<u64>,
    path: Option<String>,
    ctime: Option<u64>,
    mtime: Option<u64>,
    isdir: Option<i32>,
    category: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FileOperationResponse {
    errno: i32,
    info: Option<Vec<FileProcessStatus>>,
    taskid: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FileProcessStatus {
    errno: i32,
    path: String,
}
