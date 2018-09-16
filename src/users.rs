// =======================================================================
//  Copyleft City:Arts Project 2018-∞.
//  Distributed under the terms of the 3-Clause BSD License.
//  (See accompanying file LICENSE or copy at
//   https://opensource.org/licenses/BSD-3-Clause)
// =======================================================================

//* Use from external library *//
use rocket::http::RawStr;
use diesel::{self, prelude::*, result};
use sha3::{Digest, Sha3_256};
use rand::prelude::*;

mod schema {
    table! {
        users {
            id -> Nullable<Integer>,
            username -> Text,
            email -> Text,
            password -> Text,
            mc_username -> Nullable<Text>,
            mc_status -> Integer, // 0 - Not register / 1 - Users / 2 - Admins / 3 - Banned
            mc_ban_msg -> Nullable<Text>,
            stars -> Integer, // 0 - Users / 1 - Admins / 2 - Developers / 3 - Banned
            user_icon -> Integer,
            user_icon_path -> Nullable<Text>,
        }
    }
}

//* Use from local library *//
use self::schema::users;
use self::schema::users::dsl::{users as all_users, username as all_username, email as all_email};
use super::User;

#[table_name="users"]
#[derive(Serialize, Queryable, Insertable, Debug, Clone)]
pub struct DBUser {
    pub id: Option<i32>,
    pub username: String,
    pub email: String,
    pub password: String,
    pub mc_username: Option<String>,
    pub mc_status: i32,
    pub mc_ban_msg: Option<String>,
    pub stars: i32,
    pub user_icon: i32,
    pub user_icon_path: Option<String>
}

#[derive(FromForm)]
pub struct Register<'r> {
    pub username: &'r RawStr,
    pub email: &'r RawStr,
    pub password: &'r RawStr,
    pub re_password: &'r RawStr
}

#[derive(FromForm)]
pub struct Login<'r> {
    pub username: &'r RawStr,
    pub password: &'r RawStr
}

#[derive(Debug)]
pub enum LoginErr {
    NotExistUser,
    WrongPassword
}

fn hash_password<D: Digest>(password: &str, salt: &str) -> String {
    let mut hasher = D::new();
    hasher.input(password.as_bytes());
    hasher.input(b"$");
    hasher.input(salt.as_bytes());

    let mut result = String::new();
    for u in hasher.result().as_slice() {
        result.push_str(&format!("{:02x}", u));
    }

    result
}

impl DBUser {
    pub fn all(conn: &SqliteConnection) -> Vec<DBUser> {
        all_users.order(users::id.desc()).load::<DBUser>(conn).unwrap()
    }

    pub fn insert(register: &Register, conn: &SqliteConnection) -> bool {
        let password_out = hash_password::<Sha3_256>(register.password, register.password);
        let user_icon = thread_rng().gen_range(0, 10);
        let u = DBUser { id: None, username: register.username.to_string(), email: register.email.to_string(), password: password_out, mc_username: None, mc_status: 0, mc_ban_msg: None, stars: 0, user_icon: user_icon, user_icon_path: None };
        diesel::insert_into(users::table).values(&u).execute(conn).is_ok()
    }

    pub fn find_by_user(user: User, conn: &SqliteConnection) -> Result<DBUser, result::Error> {
        all_users.find(user.0 as i32).get_result::<DBUser>(conn)
    }

    pub fn find_by_username(username: String, conn: &SqliteConnection) -> Result<DBUser, result::Error> {
        all_users.filter(all_username.like(username)).get_result::<DBUser>(conn)
    }

    pub fn find_by_email(email: String, conn: &SqliteConnection) -> Result<DBUser, result::Error> {
        all_users.filter(all_email.like(email)).get_result::<DBUser>(conn)
    }

    pub fn do_login(login: &Login, conn: &SqliteConnection) -> Result<DBUser, LoginErr> {
        let mut user = Self::find_by_username(login.username.to_string(), conn);
        if user.is_err() { 
            user = Self::find_by_email(login.username.to_string(), conn);
            if user.is_err() { return Err(LoginErr::NotExistUser) }
        }

        let user_unwrap = user.unwrap();
        if hash_password::<Sha3_256>(login.password, login.password) != user_unwrap.password { return Err(LoginErr::WrongPassword) }
        
        Ok(user_unwrap)
    }

    pub fn delete_with_id(id: i32, conn: &SqliteConnection) -> bool {
        diesel::delete(all_users.find(id)).execute(conn).is_ok()
    }
}