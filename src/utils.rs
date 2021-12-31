use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use poem::Request;
use poem_openapi::payload::Json;
use poem_openapi::types::{ParseError, ParseFromJSON, ParseResult, ToJSON, Type};
use poem_openapi::{ApiResponse, SecurityScheme};
use poem_openapi::auth::Bearer;
use poem_openapi::registry::MetaSchemaRef;
use scylla::cql_to_rust::{FromCqlVal, FromCqlValError};
use scylla::frame::response::result::CqlValue;
use serde_json::{json, Value};


pub struct JsSafeBigInt(pub i64);

impl Deref for JsSafeBigInt {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for JsSafeBigInt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

impl Type for JsSafeBigInt {
    const IS_REQUIRED: bool = <i64 as Type>::IS_REQUIRED;
    type RawValueType = <i64 as Type>::RawValueType;
    type RawElementValueType = <i64 as Type>::RawElementValueType;

    fn name() -> Cow<'static, str> {
        Cow::from("DiscordId")
    }

    fn schema_ref() -> MetaSchemaRef {
        i64::schema_ref()
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(&self.0)
    }

    fn raw_element_iter<'a>(&'a self) -> Box<dyn Iterator<Item=&'a Self::RawElementValueType> + 'a> {
        self.0.raw_element_iter()
    }
}

impl ToJSON for JsSafeBigInt {
    fn to_json(&self) -> Value {
        json!(self.0.to_string())
    }
}

impl ParseFromJSON for JsSafeBigInt {
    fn parse_from_json(value: Value) -> ParseResult<Self> {
        value.as_i64()
            .map(|v| Self(v))
            .ok_or_else(|| ParseError::custom("cannot convert value into integer"))
    }
}

impl FromCqlVal<CqlValue> for JsSafeBigInt {
    fn from_cql(cql_val: CqlValue) -> Result<Self, FromCqlValError> {
        cql_val.as_bigint()
            .map(|v| Self(v))
            .ok_or_else(|| FromCqlValError::BadCqlType)
    }
}


lazy_static!{
    static ref SUPERUSER_KEY: Option<String> = {
      std::env::var("SUPERUSER_KEY").ok()
    };
}

#[derive(SecurityScheme)]
#[oai(type = "bearer")]
pub struct TokenBearer(pub Bearer);

#[derive(SecurityScheme)]
#[oai(type = "bearer", checker = "token_checker")]
pub struct SuperUserBearer(());

async fn token_checker(_: &Request, bearer: Bearer) -> Option<()> {
    if let Some(key) = SUPERUSER_KEY.as_ref() {
        if &bearer.token == key {
            return Some(())
        }
    }

    None
}

#[derive(ApiResponse)]
pub enum JsonResponse<T: Send + Sync + ToJSON> {
    #[oai(status = 200)]
    Ok(Json<T>),

    #[oai(status = 400)]
    BadRequest(Json<Value>),

    #[oai(status = 401)]
    Unauthorized,

    #[oai(status = 403)]
    Forbidden,
}
