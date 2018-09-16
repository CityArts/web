// =======================================================================
//  Copyleft City:Arts Project 2018-∞.
//  Distributed under the terms of the 3-Clause BSD License.
//  (See accompanying file LICENSE or copy at
//   https://opensource.org/licenses/BSD-3-Clause)
// =======================================================================

//* Use from external library *//
use std::mem;
use std::collections::HashMap;
use rocket::outcome::IntoOutcome;
use rocket::request::{self, Form, FromRequest, Request}; // Temp : FlashMessage, From
use rocket::response::{Flash, Redirect}; // Temp : Flash
use rocket::http::{Cookie, Cookies}; // Temp : Cookie
use rocket_contrib::Template;

//* Use from local library *//
use super::{TemplateContext, User, DbConn};
use users::{Register, Login, LoginErr, DBUser};

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<User, ()> {
        request.cookies()
            .get_private("user_id")
            .and_then(|cookie| cookie.value().parse().ok())
            .map(|id| User(id))
            .or_forward(())
    }
}

#[get("/accounts", rank = 2)]
fn index() -> Redirect {
    Redirect::to("/accounts/login")
}

#[get("/accounts")]
fn user_index(user: User, conn: DbConn) -> Template {
    let context = TemplateContext::new(DBUser::find_by_user(user, &conn).unwrap_or(unsafe { mem::zeroed() }), "/accounts");
    Template::render("accounts/index", &context)
}

#[get("/accounts/login")]
fn login(_user: User) -> Redirect {
    Redirect::to("/accounts")
}

#[get("/accounts/login", rank = 2)]
fn user_login(mut cookies: Cookies, conn: DbConn) -> Template {
    let context = TemplateContext::new(DBUser::find_by_user(User(0), &conn).unwrap_or(unsafe { mem::zeroed() }), "/accounts");
    Template::render("accounts/login", &context)
}

#[post("/accounts/login", data = "<login_form>")]
fn get_login<'a>(mut cookies: Cookies, login_form: Form<'a, Login<'a>>, conn: DbConn) -> Flash<Redirect> {
    let login = login_form.get();
    let do_login = DBUser::do_login(login, &conn);

    if do_login.is_err() {
        match do_login.unwrap_err() {
            LoginErr::NotExistUser => Flash::error(Redirect::to("/accounts/login"), "Username is unmatched."),
            LoginErr::WrongPassword => Flash::error(Redirect::to("/accounts/login"), "Password iss wrong.")
        }
    } else {
        let do_login_unwrap = do_login.unwrap();
        cookies.add_private(Cookie::new("user_id", do_login_unwrap.id.unwrap().to_string()));
        Flash::success(Redirect::to("/accounts"), "Success!")
    }
}

#[get("/accounts/logout")]
fn logout(mut cookies: Cookies) -> Flash<Redirect> {
    cookies.remove_private(Cookie::named("user_id"));
    Flash::success(Redirect::to("/accounts/login"), "Successfully logged out.")
}

#[get("/accounts/register")]
fn user_register(mut cookies: Cookies, conn: DbConn) -> Template {
    let context = TemplateContext::new(DBUser::find_by_user(User(0), &conn).unwrap_or(unsafe { mem::zeroed() }), "/accounts");
    Template::render("accounts/register", &context)
}

#[post("/accounts/register", data = "<register_form>")]
fn get_register<'a>(mut cookies: Cookies, register_form: Form<'a, Register<'a>>, conn: DbConn) -> Flash<Redirect> {
    let register = register_form.get();
    if register.password != register.re_password {
        Flash::error(Redirect::to("/accounts/register"), "Password is unmatched.")
    } else if DBUser::find_by_username(register.username.to_string(), &conn).is_ok() {
        Flash::error(Redirect::to("/accounts/register"), "Username is already used.")
    } else if DBUser::insert(register, &conn) {
        Flash::success(Redirect::to("/accounts/login"), "Success!")
    } else {
        Flash::error(Redirect::to("/accounts/register"), "Whoops! The server failed.")
    }
}

