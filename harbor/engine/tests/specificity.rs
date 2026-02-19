use harbor::{
    css::{
        selectors::{Specificity, parse_tokens_as_selector_list},
        tokenize::tokenize,
    },
    infra::InputStream,
};

fn specificity_of(raw_selector: &str) -> (u32, u32, u32) {
    let tokens = tokenize(&mut InputStream::new(
        &raw_selector.chars().collect::<Vec<char>>()[..],
    ));

    println!("tokens: {:#?}", tokens);
    let selector = parse_tokens_as_selector_list(tokens).unwrap();
    println!("sekector: {:#?}", selector);

    assert!(selector.len() == 1);
    selector[0].specificity()
}

#[test]
fn test_specificity_01() {
    assert_eq!(specificity_of("ul#nav li.active a"), (1, 1, 3));
}

#[test]
fn test_specificity_02() {
    assert_eq!(specificity_of("body.ie7 .col_3 h2 ~ h2"), (0, 2, 3));
}

#[test]
fn test_specificity_03() {
    assert_eq!(specificity_of("#footer *:not(nav) li"), (1, 0, 2));
}

#[test]
fn test_specificity_04() {
    assert_eq!(specificity_of("*"), (0, 0, 0));
}

#[test]
fn test_specificity_05() {
    assert_eq!(specificity_of("li"), (0, 0, 1));
}

#[test]
fn test_specificity_06() {
    assert_eq!(specificity_of("ul li"), (0, 0, 2));
}

#[test]
fn test_specificity_07() {
    assert_eq!(specificity_of("ul ol+li"), (0, 0, 3));
}

#[test]
fn test_specificity_08() {
    assert_eq!(specificity_of("h1 + *[rel=up]"), (0, 1, 1));
}

#[test]
fn test_specificity_09() {
    assert_eq!(specificity_of("ul ol li.red"), (0, 1, 3));
}

#[test]
fn test_specificity_10() {
    assert_eq!(specificity_of("li.red.level"), (0, 2, 1));
}
