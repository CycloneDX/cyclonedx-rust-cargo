/*
 * This file is part of CycloneDX Rust Cargo.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * SPDX-License-Identifier: Apache-2.0
 */
use std::{io, str::FromStr};

use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use xml_writer::XmlWriter;

use crate::traits::{ToXml};

lazy_static! {
    static ref EMAIL_REGEX: Regex = Regex::new(r#"(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9]))\.){3}(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9])|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])"#).unwrap();
}

#[derive(Debug)]
pub enum AuthorParseError {
    Email,
}

#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Author {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
}

impl Author {
    fn new(name: String, email: Option<String>) -> Result<Self, AuthorParseError> {
        if email
            .as_ref()
            .map(|should_be_email| EMAIL_REGEX.is_match(should_be_email))
            == Some(false)
        {
            return Err(AuthorParseError::Email);
        }

        Ok(Self { name, email })
    }
}

impl ToXml for Vec<Author> {
    fn to_xml<W: io::Write>(&self, xml: &mut XmlWriter<W>) -> io::Result<()> {
        xml.begin_elem("authors")?;

        for author in self {
            author.to_xml(xml)?;
        }

        xml.end_elem()
    }
}

impl FromStr for Author {
    type Err = AuthorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.ends_with('>') {
            let mut parts = s[..s.len() - 1].rsplitn(2, '<');
            let maybe_email = parts.next();
            let maybe_name = parts.next();

            let email = maybe_email.map(String::from);

            if let Some(name) = maybe_name {
                Self::new(name.trim().to_string(), email)
            } else {
                Self::new(s.to_string(), email)
            }
        } else {
            Self::new(s.to_string(), None)
        }
    }
}

impl ToXml for Author {
    fn to_xml<W: io::Write>(&self, xml: &mut XmlWriter<W>) -> io::Result<()> {
        xml.begin_elem("author")?;

        xml.elem_text("name", &self.name)?;

        if let Some(email) = &self.email {
            xml.elem_text("email", email)?;
        }

        xml.end_elem()
    }
}

#[cfg(test)]
mod test {
    use super::Author;

    fn author(name: &'static str, email: impl Into<Option<&'static str>>) -> Author {
        Author::new(name.into(), email.into().map(String::from)).unwrap()
    }

    #[test]
    fn name_with_email() {
        assert_eq!(
            author("Steve Springett", "steve.springett@owasp.org"),
            "Steve Springett <steve.springett@owasp.org>"
                .parse()
                .unwrap()
        );
    }

    #[test]
    fn author_without_email() {
        assert_eq!(
            author("Steve Springett", None),
            "Steve Springett".parse().unwrap()
        );
    }
}
