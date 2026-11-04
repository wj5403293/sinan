use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("配置解析失败：{0} 请尝试检查或删除配置文件")]
    ConfigurationParsingFailed(String),

    #[error("配置生成失败：{0} 请尝试检查或删除配置文件")]
    ConfigurationGenerationFailed(String),

    #[error("输入不能为空！")]
    EmptyInput,

    #[error("包含无效的十六进制地址！请输入正确的格式。")]
    InvalidHex,

    #[error("解析失败，请输入有效的正整数！")]
    InvalidPositiveInteger,

    #[error("配置列表找不到目标：{0}")]
    TargetNotFound(String),

    #[error("未找到 PID 为 {0} 的活动进程！")]
    ProcessNotFound(u32),

    #[error("进程目标未设置")]
    TargetNotSet,

    #[error("无法获取进程句柄，请确保进程正在运行")]
    ProcessHandleNotFound,
}

impl From<AppError> for mlua::Error {
    fn from(err: AppError) -> Self {
        mlua::Error::external(err)
    }
}
