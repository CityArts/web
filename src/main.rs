// =======================================================================
//  Copyleft City:Arts Project 2018-∞.
//  Distributed under the terms of the 3-Clause BSD License.
//  (See accompanying file LICENSE or copy at
//   https://opensource.org/licenses/BSD-3-Clause)
// =======================================================================

#![feature(plugin, decl_macro, never_type, custom_attribute)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;
#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;
extern crate rocket_contrib;
extern crate sha3;
extern crate rand;

mod accounts;
mod forum;
mod users;

//* Use from external library *//
use std::mem;
use std::path::{PathBuf, Path};
use rocket::{Rocket, Request};
use rocket::fairing::AdHoc;
use rocket::response::NamedFile; // Temp : Redirect
use rocket_contrib::{Template, databases::database};
use diesel::SqliteConnection;

//* Use from local library *//
use users::DBUser;

embed_migrations!();

#[database("sqlite_database")]
pub struct DbConn(SqliteConnection);

#[derive(Serialize)]
pub struct TemplateContext {
    pub is_login: bool,
    pub user: DBUser,
    pub current_url: &'static str,
    pub user_icon_str: &'static str,
    pub first_username: String,
}

#[derive(Debug, Serialize)]
pub struct User(usize);

impl TemplateContext {
    pub fn new(user: DBUser, current_url: &'static str) -> Self {
        let is_login = !user.id.is_none();
        let user_icon_str = Self::user_icon_to_str(user.user_icon);
        let first_username = {
            let mut first = String::new();
            first.push(user.username.chars().nth(0).unwrap_or('L'));
            first
        };

        Self {
            is_login: is_login,
            user: user,
            current_url: current_url,
            user_icon_str: user_icon_str,
            first_username: first_username.to_uppercase()
        }
    }

    fn user_icon_to_str(user_icon: i32) -> &'static str {
        match user_icon {
            1 => "night_fade",
            2 => "spring_warmth",
            3 => "sunny_morning",
            4 => "lady_lips",
            5 => "rainy_ashville",
            6 => "winter_neva",
            7 => "heavy_rain",
            8 => "mean_fruit",
            9 => "new_life",
            10 => "wild_apple",
            _ => "rainy_ashville"
        }
    }
}

#[get("/assets/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("assets/").join(file)).ok()
}

#[get("/")]
fn index(user: Option<User>, conn: DbConn) -> Template {
    let context = TemplateContext::new(DBUser::find_by_user(user.unwrap_or(User(0)), &conn).unwrap_or(unsafe { mem::zeroed() }), "/");

    Template::render("index", &context)
}

#[catch(404)]
fn not_found(req: &Request) -> Template {
    let mut map = std::collections::HashMap::new();
    map.insert("path", req.uri().to_string());
    Template::render("error/404", &map)
}

fn main() {
    rocket().0.launch();
}

fn rocket() -> (Rocket, Option<DbConn>) {
    let rocket = rocket::ignite()
        .attach(DbConn::fairing())
        .attach(AdHoc::on_attach("Database Migrations", |rocket| {
            let conn = DbConn::get_one(&rocket).expect("database connection");
            match embedded_migrations::run(&*conn) {
                Ok(()) => Ok(rocket),
                Err(e) => {
                    error!("Failed to run database migrations: {:?}", e);
                    Err(rocket)
                },
            }
        }))
        .mount("/", routes![index, accounts::index, accounts::user_index, accounts::login, accounts::user_login, accounts::user_register, accounts::get_register, accounts::get_login, accounts::logout, files])
        .attach(Template::fairing());

    let conn = DbConn::get_one(&rocket);

    (rocket, conn)
}
