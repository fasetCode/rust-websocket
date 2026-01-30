
#[derive(Debug, serde::Serialize)]
pub struct PageVo<T>{
    // 总数
    pub total: i64,
    // 数据 list Obj
    pub list: Vec<T>,
}

impl<T> PageVo<T>{
    pub fn new(total: i64, list: Vec<T>) -> Self{
        PageVo{
            total,
            list,
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct ResultVo<T>{
    pub code: i32,
    pub msg: String,
    pub data: Option<T>,
}

impl<T> ResultVo<T> {
    // pub fn ok() -> Self{
    //     ResultVo{
    //         code: 0,
    //         msg: "success".to_string(),
    //         data: None,
    //     }
    // }
    pub fn ok_with(data: T) -> Self{
        ResultVo{
            code: 0,
            msg: "success".to_string(),
            data: Some(data),
        }
    }
    pub fn error(code: i32, msg: String) -> Self{
        ResultVo{
            code,
            msg,
            data: None,
        }
    }
}