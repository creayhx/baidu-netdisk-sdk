/// Baidu NetDisk API Error Code Mapping
///
/// Reference: https://pan.baidu.com/union/doc/okumlx17r
///
/// This module provides human-readable descriptions for common API error codes.
/// Error messages are kept in Chinese to match the official API responses.
use std::collections::HashMap;
use std::sync::OnceLock;

/// Error code description mapping
static ERROR_CODES: OnceLock<HashMap<i32, &'static str>> = OnceLock::new();

fn get_error_codes() -> &'static HashMap<i32, &'static str> {
    ERROR_CODES.get_or_init(|| {
        let mut map = HashMap::new();

        // Common error codes
        map.insert(0, "成功");
        map.insert(-1, "系统错误");
        map.insert(-2, "参数错误");
        map.insert(-3, "系统内部错误");
        map.insert(-4, "验证失败");
        map.insert(-5, "权限不足");
        map.insert(-6, "文件不存在");
        map.insert(-7, "目录不存在");
        map.insert(-8, "已存在");
        map.insert(-9, "不允许的操作");
        map.insert(-10, "配额超限");
        map.insert(-11, "空间不足");
        map.insert(-12, "无效路径");
        map.insert(-13, "文件数量过多");
        map.insert(-14, "文件过大");
        map.insert(-15, "无效文件名");
        map.insert(-16, "无法修改文件");
        map.insert(-17, "无法删除目录");
        map.insert(-18, "无法创建目录");
        map.insert(-19, "不支持的文件类型");
        map.insert(-20, "操作超时");
        map.insert(-21, "网络错误");
        map.insert(-22, "无效token");
        map.insert(-23, "token已过期");
        map.insert(-24, "无效会话");
        map.insert(-25, "用户未登录");
        map.insert(-26, "用户已暂停");
        map.insert(-27, "服务不可用");
        map.insert(-28, "无效请求");
        map.insert(-29, "请求过于频繁");
        map.insert(-30, "访问被拒绝");

        // File operation error codes
        map.insert(1, "文件已存在");
        map.insert(2, "参数错误");
        map.insert(3, "无效路径");
        map.insert(4, "权限不足");
        map.insert(5, "系统错误");
        map.insert(6, "文件不存在");
        map.insert(7, "目录不存在");
        map.insert(8, "已存在");
        map.insert(9, "无法创建");
        map.insert(10, "无法删除");
        map.insert(11, "无法重命名");
        map.insert(12, "无法移动");
        map.insert(13, "无法复制");
        map.insert(14, "空间不足");
        map.insert(15, "配额超限");
        map.insert(16, "文件过大");
        map.insert(17, "无效文件名");
        map.insert(18, "无效文件类型");
        map.insert(19, "文件数量过多");
        map.insert(20, "操作超时");

        // Authentication error codes
        map.insert(100, "无效API密钥");
        map.insert(101, "无效密钥");
        map.insert(102, "无效授权类型");
        map.insert(103, "无效授权码");
        map.insert(104, "无效刷新token");
        map.insert(105, "无效访问token");
        map.insert(106, "token已过期");
        map.insert(107, "token已撤销");
        map.insert(108, "不允许的作用域");
        map.insert(109, "用户未授权");
        map.insert(110, "无效重定向URI");
        map.insert(111, "授权码已过期");

        // Upload/download error codes
        map.insert(200, "上传失败");
        map.insert(201, "下载失败");
        map.insert(202, "上传文件过大");
        map.insert(203, "下载文件过大");
        map.insert(204, "上传超时");
        map.insert(205, "下载超时");
        map.insert(206, "上传中断");
        map.insert(207, "下载中断");
        map.insert(208, "无效上传ID");
        map.insert(209, "无效分块编号");
        map.insert(210, "ETag不匹配");

        // Share error codes
        map.insert(300, "分享链接无效");
        map.insert(301, "分享链接已过期");
        map.insert(302, "分享链接需要密码");
        map.insert(303, "无效分享密码");
        map.insert(304, "分享权限不足");
        map.insert(305, "无法分享文件");
        map.insert(306, "无法取消分享");
        map.insert(307, "分享数量过多");

        // Third-party application error codes
        map.insert(31001, "应用不存在");
        map.insert(31002, "应用已禁用");
        map.insert(31003, "应用未授权");
        map.insert(31004, "应用配额超限");
        map.insert(31005, "应用请求过于频繁");

        // Stream-related error codes
        map.insert(31301, "非流式文件");
        map.insert(31302, "流式文件不存在");
        map.insert(31303, "流式文件已过期");
        map.insert(31304, "流式文件权限不足");

        // Download error codes
        map.insert(
            31045,
            "access_token验证未通过，请检查access_token是否过期，用户授权时是否勾选网盘权限等。",
        );
        map.insert(31296, "服务器内部错误，请稍后重试。");
        map.insert(31362, "签名错误，请检查链接地址是否完整。");
        map.insert(31326, "命中防盗链，需检查User-Agent请求头是否正常。");
        map.insert(31360, "链接过期。dlink有效期为8小时");

        map
    })
}

/// Get error description for a specific error code
pub fn get_error_description(errno: i32) -> Option<&'static str> {
    get_error_codes().get(&errno).copied()
}

/// Get full error message for an error code
pub fn get_error_message(errno: i32, default_message: &str) -> String {
    match get_error_description(errno) {
        Some(desc) => format!("{} (errno: {})", desc, errno),
        None => format!("{} (errno: {})", default_message, errno),
    }
}

/// Check if error code indicates success
pub fn is_success(errno: i32) -> bool {
    errno == 0
}

/// Check if error code indicates authentication failure
pub fn is_auth_error(errno: i32) -> bool {
    (100..=111).contains(&errno) || errno == -22 || errno == -23 || errno == -24 || errno == -25
}

/// Check if error code indicates file not found
pub fn is_not_found_error(errno: i32) -> bool {
    errno == -6 || errno == -7 || errno == 6 || errno == 7
}

/// Check if error code indicates permission issue
pub fn is_permission_error(errno: i32) -> bool {
    errno == -5 || errno == -30 || errno == 4 || errno == 304
}

/// Check if error code indicates quota issue
pub fn is_quota_error(errno: i32) -> bool {
    errno == -10 || errno == -11 || errno == 14 || errno == 15
}

/// Check if error code indicates token expiration
pub fn is_token_expired(errno: i32) -> bool {
    errno == -23 || errno == 106
}
