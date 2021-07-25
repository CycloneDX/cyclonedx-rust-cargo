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
 * Copyright (c) OWASP Foundation. All Rights Reserved.
 */
use std::io::{self, Write};

use xml_writer::XmlWriter;

/// Write a CycloneDX XML representation of `self`.
pub trait ToXml {
    /// Write a CycloneDX XML representation of `self`.
    ///
    /// # Requirements
    /// * If `to_xml` returns `Ok`, then `xml` must have the same tag depth as it did when the
    ///   function was invoked.
    fn to_xml<W: Write>(&self, xml: &mut XmlWriter<W>) -> io::Result<()>;
}

/// Check if `self` contains meaningful data.
pub trait IsEmpty {
    /// Check if `self` contains meaningful data.
    fn is_empty(&self) -> bool;
}

impl<'a> IsEmpty for &'a str {
    fn is_empty(&self) -> bool {
        !self.trim().is_empty()
    }
}

impl IsEmpty for String {
    fn is_empty(&self) -> bool {
        self.as_str().is_empty()
    }
}

impl<T: IsEmpty> IsEmpty for Option<T> {
    fn is_empty(&self) -> bool {
        if let Some(v) = self {
            v.is_empty()
        } else {
            true
        }
    }
}

impl<T> IsEmpty for Vec<T> {
    fn is_empty(&self) -> bool {
        Vec::<T>::is_empty(self)
    }
}
