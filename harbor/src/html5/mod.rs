#![allow(dead_code)]

pub mod dom;
/// Custom implementation of the HTML5 spec:
/// https://html.spec.whatwg.org/
pub mod parse;

macro_rules! concat_arrays {
    ( $ty:ty, $default:expr => $($arr:expr),* $(,)? ) => {{
        const __CONCAT_ARRAYS_LEN: usize = 0 $( + $arr.len() )*;
        const __CONCAT_ARRAYS_RESULT: [$ty; __CONCAT_ARRAYS_LEN] = {
            let mut result = [$default; __CONCAT_ARRAYS_LEN];
            let mut result_idx = 0;
            $(
                let arr = $arr;
                let mut src_idx = 0;
                while src_idx < arr.len() {
                    result[result_idx] = arr[src_idx];
                    src_idx += 1;
                    result_idx += 1;
                }
            )*
            result
        };
        __CONCAT_ARRAYS_RESULT
    }};
}

pub const HTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";

pub mod tag_groups {
    pub const DEFAULT_SCOPE_NAMES: [&str; 14] = [
        "applet", "caption", "html", "table", "td", "th", "marquee", "object", "template", "mi",
        "mo", "mn", "ms", "mtext",
    ];

    pub const BUTTON_SCOPE_NAMES: [&str; 15] =
        concat_arrays!(&str, "" => &DEFAULT_SCOPE_NAMES, &["button"]);

    pub const LIST_ITEM_SCOPE_NAMES: [&str; 16] =
        concat_arrays!(&str, "" => &DEFAULT_SCOPE_NAMES, &["ol", "ul"]);

    pub const IMPLIED_END_TAGS: [&str; 10] = [
        "dd", "dt", "li", "option", "optgroup", "p", "rb", "rp", "rt", "rtc",
    ];

    pub const FORMATTING_ELEMENT_NAMES: [&str; 12] = [
        "b", "big", "code", "em", "font", "i", "s", "small", "strike", "strong", "tt", "u",
    ];

    pub const SPECIAL_CATEGORY_NAMES: [&str; 83] = [
        "address",
        "applet",
        "area",
        "article",
        "aside",
        "base",
        "basefont",
        "bgsound",
        "blockquote",
        "body",
        "br",
        "button",
        "caption",
        "center",
        "col",
        "colgroup",
        "dd",
        "details",
        "dir",
        "div",
        "dl",
        "dt",
        "embed",
        "fieldset",
        "figcaption",
        "figure",
        "footer",
        "form",
        "frame",
        "frameset",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "head",
        "header",
        "hgroup",
        "hr",
        "html",
        "iframe",
        "img",
        "input",
        "keygen",
        "li",
        "link",
        "listing",
        "main",
        "marquee",
        "menu",
        "meta",
        "nav",
        "noembed",
        "noframes",
        "noscript",
        "object",
        "ol",
        "p",
        "param",
        "plaintext",
        "pre",
        "script",
        "search",
        "section",
        "select",
        "source",
        "style",
        "summary",
        "table",
        "tbody",
        "td",
        "template",
        "textarea",
        "tfoot",
        "th",
        "thead",
        "title",
        "tr",
        "track",
        "ul",
        "wbr",
        "xmp",
    ];

    pub const ARBITRARY_SPECIAL_GROUP_END: [&str; 28] = [
        "address",
        "article",
        "aside",
        "blockquote",
        "button",
        "center",
        "details",
        "dialog",
        "dir",
        "div",
        "dl",
        "fieldset",
        "figcaption",
        "figure",
        "footer",
        "header",
        "hgroup",
        "listing",
        "main",
        "menu",
        "nav",
        "ol",
        "pre",
        "search",
        "section",
        "select",
        "summary",
        "ul",
    ];

    pub const ARBITRARY_SPECIAL_GROUP_START: [&str; 25] = [
        "address",
        "article",
        "aside",
        "blockquote",
        "center",
        "details",
        "dialog",
        "dir",
        "div",
        "dl",
        "fieldset",
        "figcaption",
        "figure",
        "footer",
        "header",
        "hgroup",
        "main",
        "menu",
        "nav",
        "ol",
        "p",
        "search",
        "section",
        "summary",
        "ul",
    ];
}
