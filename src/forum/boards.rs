// =======================================================================
//  Copyleft City:Arts Project 2018-∞.
//  Distributed under the terms of the 3-Clause BSD License.
//  (See accompanying file LICENSE or copy at
//   https://opensource.org/licenses/BSD-3-Clause)
// =======================================================================

//* Use from external library *//
use diesel::{self, prelude::*, result};

mod schema {
    table! {
        boards {
            id -> Nullable<Integer>,
            name -> Text,
            url -> Text,
            write_stars -> Integer,
            read_stars -> Integer,
        }
    }
}

//* Use from local library *//
use self::schema::boards;
use self::schema::users::dsl::{boards as all_boards};

#[table_name="boards"]
#[derive(Serialize, Queryable, Insertable, Debug, Clone)]
pub struct DBUser {
    pub id: Option<i32>,
    pub name: String,
    pub url: String,
    pub write_stars: i32,
    pub read_stars: i32
}