/*
 * Copyright (c) 2020, Мира Странная <rsxrwscjpzdzwpxaujrr@yahoo.com>
 *
 * This program is free software: you can redistribute it and/or
 * modify it under the terms of the GNU Affero General Public License
 * as published by the Free Software Foundation, either version 3 of
 * the License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::error::Error;
use serde::Deserialize;
use tera::Context;
use actix_web::{ HttpRequest, HttpMessage, HttpResponse, cookie::Cookie, web, http::header };

use crate::errors::*;
use crate::state::State;

pub struct Auth<'a> {
    token: String,
    cookie: Cookie<'a>,
}

impl Auth<'_> {
    pub fn new(token: String) -> Result<Auth<'static>, Box<dyn Error>> {
        Auth::check_token(token.as_str())?;

        Ok(Auth { token, cookie: Cookie::named("auth") })
    }

    pub fn authorized(&self, req: &HttpRequest) -> bool {
        match req.cookie("auth") {
            Some(cookie) => { cookie.value() == self.token }
            _ => { false }
        }
    }

    pub fn auth(&mut self, token: String) -> bool {
        if token == self.token {
            self.cookie.set_value(token);
            return true;
        }

        return false;
    }

    pub fn deauth(&self, response: &mut HttpResponse) {
        response.add_cookie(&Cookie::named("auth"));
    }

    pub fn cookie(&self) -> &Cookie {
        &self.cookie
    }

    fn check_token(token: &str) -> Result<(), Box<dyn Error>> {
        if !token.is_ascii() {
            return Err("Token should be ascii string".into());
        }

        if !token.len() < 32 {
            return Err("Token length should be over 32".into());
        }

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct AuthFormData {
    token: String,
}

pub async fn auth_submit(req: HttpRequest,
                         state: web::Data<State<'_>>,
                         form: web::Form<AuthFormData>) -> HttpResponse {
    let mut response = HttpResponse::SeeOther()
        .header("Location", "/")
        .finish();

    let mut auth = try_500!(state.auth.lock(), state, req);

    if auth.auth(form.token.clone()) {
        try_500!(response.add_cookie(auth.cookie()), state, req);
    }

    response
}

pub async fn auth(req: HttpRequest,
                  state: web::Data<State<'_>>) -> HttpResponse {
    let mut context = Context::new();
    let auth = try_500!(state.auth.lock(), state, req);

    context.insert("authorized", &auth.authorized(&req));

    return HttpResponse::Ok().body(try_500!(state.tera.render("auth.html", &context), state, req));
}

pub async fn deauth(req: HttpRequest,
                    state: web::Data<State<'_>>) -> HttpResponse {
    let mut url = "/";

    if let Some(temp_url) = req.headers().get(header::REFERER) {
        if let Ok(temp_url) = temp_url.to_str() {
            url = temp_url;
        }
    }

    let mut response = HttpResponse::SeeOther()
        .header("Location", url)
        .finish();

    let auth = try_500!(state.auth.lock(), state, req);

    auth.deauth(&mut response);

    response
}
