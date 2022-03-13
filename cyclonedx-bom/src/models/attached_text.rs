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

use crate::external_models::normalized_string::NormalizedString;

#[derive(Debug, PartialEq)]
pub struct AttachedText {
    pub(crate) content_type: Option<NormalizedString>,
    pub(crate) encoding: Option<Encoding>,
    pub(crate) content: String,
}

impl AttachedText {
    /// Construct a new `AttachedText`
    ///
    /// - `content_type` - Content type of the attached text (default: `"text/plain"`)
    /// - `content` - Raw content, which will be base64 encoded when added to the BOM
    pub fn new<T: AsRef<[u8]>>(content_type: Option<NormalizedString>, content: T) -> Self {
        Self {
            content_type,
            encoding: Some(Encoding::Base64),
            content: base64::encode(content),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum Encoding {
    Base64,
    #[doc(hidden)]
    UnknownEncoding(String),
}

impl ToString for Encoding {
    fn to_string(&self) -> String {
        match self {
            Encoding::Base64 => "base64".to_string(),
            Encoding::UnknownEncoding(ue) => ue.clone(),
        }
    }
}

impl Encoding {
    pub(crate) fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "base64" => Self::Base64,
            unknown => Self::UnknownEncoding(unknown.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::models::attached_text::Encoding;

    use super::*;

    #[test]
    fn it_should_construct_attached_text() {
        let actual = AttachedText::new(
            Some(NormalizedString::new("text/plain")),
            "this text is plain",
        );
        assert_eq!(
            actual,
            AttachedText {
                content_type: Some(NormalizedString::new("text/plain")),
                encoding: Some(Encoding::Base64),
                content: "dGhpcyB0ZXh0IGlzIHBsYWlu".to_string(),
            }
        )
    }
}
