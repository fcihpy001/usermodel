use std::time;
use std::time::Duration;
use actix_web::{HttpResponse, Responder, error, Error, HttpRequest};
use actix_web::web::{Data, Form, Path};
use chrono::NaiveDate;
use jsonwebtoken::{decode, DecodingKey, encode, EncodingKey, Header, Validation, errors::ErrorKind, Algorithm, decode_header};
use log::{error, info};
use mysql::{ Pool};
use mysql::prelude::Queryable;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{MySql, MySqlPool};
use crate::models::{Login, LoginForm, Person, PersonForm, Resp,AppConfig};
use crate::utils::{get_config, get_jwt_secret};

pub async fn phone_login(
    form: Form<LoginForm>,
    redis: Data<redis::Client>,
    pool: Data<MySqlPool>) -> impl Responder {

    // 从redis中判断用户是否存在
    let mut  redis_conn = redis
        .get_tokio_connection_manager()
        .await
        .map_err(error::ErrorInternalServerError).unwrap();
    let res = redis::Cmd::get(&[form.phone.clone()])
        .query_async::<_, usize>(&mut redis_conn)
        .await
        .map_err(error::ErrorInternalServerError);
    match res {
        Ok(info) => {
            println!("redis {}", info);
        }
        Err(_) => {
            println!("没找到相应数据")
        }
    }

    // 判断数据库是否存在此用户
    let person = sqlx::query_as::<_,Person>("select name,age,id,phone from person where phone =?")
        .bind(form.phone.clone())
        .fetch_one(&**pool)
        .await;
    let mut uid = 0 as u32;
    let mut desc = "登录成功";
    match person {
        Ok(p) => {
            println!("11");
            uid = p.id;
        }
        Err(_) => {
            // 注册流程
            let r = rand::random::<u8>();
            let name = format!("2844用户{}",r);
            let age = 0;

            let id = sqlx::query(r#"INSERT INTO person (name, phone,age) VALUES (?,?,?)"#)
                .bind(name)
                .bind(form.phone.clone())
                .bind(age)
                .execute(&**pool)
                .await
                .unwrap()
                .last_insert_id();

            // 往redis中存一份登录信息
            let res = redis::Cmd::set(form.phone.clone(), uid)
                .query_async::<_, String>(&mut redis_conn)
                .await
                .map_err(error::ErrorInternalServerError).unwrap();
            uid = id as u32;
            desc = "注册成功";
        }
    }

    let claims = Claims {
        uid,
        sub: "2844go".to_owned(),
        exp: 10000000000
    };
    // 根据用户信息产生token
    let token_res = encode(
      &Header::default(),
        &claims,
        &EncodingKey::from_secret(get_jwt_secret().as_ref())
    );
    match token_res {
        Ok(token) => {
            // 把token信息存入redis
            let token_redis_res = redis::Cmd::set(&token, uid)
                .query_async::<_, usize>(&mut redis_conn)
                .await
                .map_err(error::ErrorInternalServerError);
            match token_redis_res {
                Ok(info) => {
                    println!("redis {}", info);
                }
                Err(_) => {
                    println!("没找到相应数据")
                }
            }

            HttpResponse::Ok().json(Resp {
                code: 0,
                msg: "Ok".to_string(),
                data: Some(json!({"uid": uid, "desc": desc, "token": token }))
            })
        }
        Err(_) => {
            HttpResponse::Ok().json(Resp {
                code: 0,
                msg: "error".to_string(),
                data: Some("token fail")
            })
        }
    }
}

#[derive(Serialize,Deserialize,Debug)]
struct Claims {
    uid: u32,
    sub: String,
    exp: usize
}

pub async fn info_query1(uid: Path<u32>, redis: Data<redis::Client>) -> impl Responder {
    // info!("info {:#?}", form.into_inner());
    let mut  conn = redis
        .get_tokio_connection_manager()
        .await
        .map_err(error::ErrorInternalServerError).unwrap();

    let res = redis::Cmd::get(&["name", "age"])
        .query_async::<_, usize>(&mut conn)
        .await
        .map_err(error::ErrorInternalServerError).unwrap();
    if res == 2 {
        HttpResponse::Ok().body("successfully deleted values")
    } else {
        log::error!("deleted {res} keys");
        HttpResponse::InternalServerError().finish()
    }
}

pub async fn info_query(id: Path<u32>,pool: Data<MySqlPool>,) -> impl Responder {
    let person = sqlx::query_as::<_,Person>("select * from person where id =?")
        .bind(*id)
        .fetch_one(&**pool)
        .await
        .unwrap();
    HttpResponse::Ok().json(Resp {
        code: 0,
        msg: "Ok".to_string(),
        data: Some(person)
    })
}

pub async fn myinfo_query(pool: Data<MySqlPool>,_req: HttpRequest) -> impl Responder {
    // 从header中获取token
    let token1 = _req.headers().get("Authorization").unwrap().to_str();
    let mut token = "";
    match token1 {
        Ok(tt) => {

            token = tt;
        }
        Err(_) => {
            println!("token_header: error");
        }
    }
    let mut uid = 0;

    let mut validation = Validation::default();
    validation.validate_exp = false;
    println!("token==: {:#?}", &token);
    match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(get_jwt_secret().as_ref()),
        &validation
    ) {
        Ok(data) => {
            uid = 21;
            println!("token对应的用户::: {:#?}",data.claims);
            uid = data.claims.uid;
        }
        Err(err) => match *err.kind() {
            ErrorKind::InvalidToken => error!("token无效"),
            ErrorKind::InvalidIssuer => error!("没有对应的用户"),
            _ => error!("other error {:?}", err),
        }
    }

    let person = sqlx::query_as::<_,Person>("select * from person where id =?")
        .bind(uid)
        .fetch_one(&**pool)
        .await
        .unwrap();
    HttpResponse::Ok().json(Resp {
        code: 0,
        msg: "Ok".to_string(),
        data: Some(person)
    })
}

pub async fn myinfo_update(form: Form<PersonForm>,pool: Data<MySqlPool>) -> impl Responder {
    let uid = 5;
    let age = &form.age.unwrap();
    // let name = &form.name.unwrap();
    let res = sqlx::query(r#"UPDATE person SET age = ? where id = ?"#)
        .bind(age)
        .bind(uid)
        .execute(&**pool)
        .await
        .unwrap();

    HttpResponse::Ok().json(Resp {
        code: 0,
        msg: "Ok".to_string(),
        data: Some("")
    })

}