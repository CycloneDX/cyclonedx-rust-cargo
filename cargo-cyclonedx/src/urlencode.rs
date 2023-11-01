//! If you are reading this - buckle up, we are going on an adventure!
//!
//! So in the purl spec there is this innocuous example of two valid PURLs:
//! ```text
//! pkg:generic/openssl@1.1.10g?download_url=https://openssl.org/source/openssl-1.1.0g.tar.gz&checksum=sha256:de4d501267da
//! pkg:generic/bitwarderl?vcs_url=git%2Bhttps://git.fsfe.org/dxtr/bitwarderl%40cc55108da32
//! ```
//! from <https://github.com/package-url/purl-spec/blob/07d2bdea6d9610b52772866c7ed7859e5566f318/PURL-TYPES.rst#generic>
//!
//! Note the `git%2Bhttps` part. The `%2B` is a percent-encoded `+` character, which is necessary, because otherwise
//! the `+` would be turned into a space when decoding and the original string would not be recoverable.
//!
//! Let's dive into the specs to see how to create such strings.
//!
//! ### WHATWG URL specification
//!
//! I have naively assumed that there is a single, well-defined percent encoding standard.
//!
//! In reality The URL spec has numerous different sets of characters that should or should not be URL-encoded.
//!
//! This part, `?foo=bar`, is called the "query" in the URL spec:
//! <https://url.spec.whatwg.org/#concept-url-query>
//!
//! And this is what characters are supposed to be URL-encoded there:
//! <https://url.spec.whatwg.org/#query-percent-encode-set>
//!
//! Note the absence of the `+` character in this set!
//! It is apparently legal to put a + in there, but the generic URL parsers I tried,
//! as well as the official JS and Go PURL implementations convert it into a space!
//!
//! There are only two character sets that escape `+`:
//! 1. <https://url.spec.whatwg.org/#component-percent-encode-set>
//!    to be used for "components", but the spec NEVER DEFINES WHAT A COMPONENT IS.
//! 2. <https://url.spec.whatwg.org/#application-x-www-form-urlencoded-percent-encode-set> to be used for form submission,
//!    [which is apparently our case](https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/523#discussion_r1378020167)?
//!
//! Both of which also escape `:`, so it's not possible to produce *both* of the valid URL examples with the same implementation -
//! at least using any of the standard character sets.
//!
//! The URL spec also includes this lovely bit:
//!
//! > This is used by HTML for registerProtocolHandler(), and could also be used by other standards
//! > to percent-encode data that can then be embedded in a URL’s path, query, or fragment; or in an opaque host.
//! > Using it with UTF-8 percent-encode gives identical results to JavaScript’s encodeURIComponent() [sic]. [HTML] [ECMA-262]
//!
//! Except it does NOT specify which of these two it refers to - component or form character set!
//!
//! ### RFC 3986 URI specification
//!
//! PURL specifies that it ALSO follows the rfc3986 spec - the URI spec:
//! <https://github.com/package-url/purl-spec/blob/master/PURL-SPECIFICATION.rst#a-purl-is-a-url>
//!
//! PURL claims it adheres to both, which is curious because the specs are subtly incompabitle.
//!
//! (They are incompatible in the way they do percent encoding too,
//! see <https://docs.rs/percent-encoding-rfc3986> which is distinct from <https://docs.rs/percent-encoding>,
//! but that's a whole other rabbit hole and I'm not going down it right now.)
//!
//! So let's see what the URI spec escapes:
//! <https://datatracker.ietf.org/doc/html/rfc3986#section-2.2>
//! Okay so `:` is super escaped and `+` is maybe escaped.
//! The official PURL examples that escape `+` but not `:` are impossible to obtain with that too!
//!
//! ### PURL specification
//!
//! PURL spec documents the process of creating a valid PURL! Maybe that will help?
//!
//! If you follow the "how to write purl" part from the spec,
//! <https://github.com/package-url/purl-spec/blob/master/PURL-SPECIFICATION.rst#how-to-build-purl-string-from-its-components>
//! it specifies that you first join the special "checksums" value together with `,` signs and do all the other stuff to it,
//! and then you percent the result (doesn't specify using which of a gazillion possible character sets).
//!
//! So according to the spec, the `checksum=sha256:de4d501267da` part in the example should not be possible to obtain!
//! It should be `checksum=sha256%3Ade4d501267da` instead!
//!
//! And even if checksums were special somehow, there is no character set that would produce `git%2Bhttps://``
//! because every single character set that escapes `+` also escapes `:`!
//!
//! The fact that the percent encoding is insufficiently defined in the PURL spec,
//! and that different PURL implementations disagree about what they escape, has been known since 2018
//! but as of this writing has not been acted upon: <https://github.com/package-url/purl-spec/issues/39>
//!
//! ### Putting it all together
//!
//! To sum up:
//!
//! 1. There are many different percent encodings
//! 2. PURL spec does not specify which one it uses
//! 3. The official PURL examples CANNOT be produced with ANY one of those standard percent encodings
//! 4. The one that `purl` crate implements for qualifiers in accordance with the WHATWG URL spec (not the rfc3986 URI spec)
//!    produces nonsensical results (does not escape `+` where it is clearly necessary) which breaks PURL parsers
//!    (but the crate authors have seen this text and [will probably do something about that](https://github.com/phylum-dev/purl/issues/11))
//!
//! So the specs failed us, and we have to rely on implementation behavior. We need to encode data so that as many decoders as possible will read it correctly.
//!
//! Percent decoders do not have a whitelist of characters they don't percent-decode, they just decode everything starting with a %.
//! That is how the spec defines percent decoding, too: <https://url.spec.whatwg.org/#percent-decode>
//! So when the PURL spec says "The value is the percent-decoded right side", the decoder should just decode everything starting with a %.
//!
//! So overdoing the encoding is not an issue; it's doing too little that's the problem.
//! Therefore, we pick the most agressive encoding out of the whole URL spec and use that:
//! <https://url.spec.whatwg.org/#application-x-www-form-urlencoded-percent-encode-set>
//! Which also happens to be compliant with the URL spec. At least we are following one of the three!
//!
//! This can be revisited if/when the PURL spec actually specifies which characters exactly have to be escaped in qualifiers.

use percent_encoding::{self, utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};

/// https://url.spec.whatwg.org/#application-x-www-form-urlencoded-percent-encode-set
const FORM_URLENCODED: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'*')
    .remove(b'-')
    .remove(b'.')
    .remove(b'_');

pub fn urlencode(s: &str) -> String {
    utf8_percent_encode(s, FORM_URLENCODED).to_string()
}
