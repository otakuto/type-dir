use super::super::{Hop, RefHead, parse_ref};

/// `${dir.x.choice.c}` — a `.choice.` hop navigates into a one_of/any_of/choice child set.
#[test]
fn choice_hop() {
    // Arrange
    let key = "dir.x.choice.c";

    // Act
    let result = parse_ref(key);

    // Assert
    assert!(result.hops.is_empty());
    match &result.head {
        RefHead::DirNs { id, tail } => {
            assert_eq!(id, "x");
            assert_eq!(tail, &vec![Hop::Choice("c".to_string())]);
        }
        other => panic!("expected DirNs, got {other:?}"),
    }
}

/// `${dir.x.group.g}` — a `.group.` hop navigates into a record-intro group child set.
#[test]
fn group_hop() {
    // Arrange
    let key = "dir.x.group.g";

    // Act
    let result = parse_ref(key);

    // Assert
    match &result.head {
        RefHead::DirNs { id, tail } => {
            assert_eq!(id, "x");
            assert_eq!(tail, &vec![Hop::Group("g".to_string())]);
        }
        other => panic!("expected DirNs, got {other:?}"),
    }
}

/// `${dir.x.for.loop1}` — a `.for.` hop navigates into a for-entry child set.
#[test]
fn for_hop() {
    // Arrange
    let key = "dir.x.for.loop1";

    // Act
    let result = parse_ref(key);

    // Assert
    match &result.head {
        RefHead::DirNs { id, tail } => {
            assert_eq!(id, "x");
            assert_eq!(tail, &vec![Hop::For("loop1".to_string())]);
        }
        other => panic!("expected DirNs, got {other:?}"),
    }
}

/// `${dir.x.fetch.dirs}` — a `.fetch.` hop navigates into a fetch-entry child set.
#[test]
fn fetch_hop() {
    // Arrange
    let key = "dir.x.fetch.dirs";

    // Act
    let result = parse_ref(key);

    // Assert
    match &result.head {
        RefHead::DirNs { id, tail } => {
            assert_eq!(id, "x");
            assert_eq!(tail, &vec![Hop::Fetch("dirs".to_string())]);
        }
        other => panic!("expected DirNs, got {other:?}"),
    }
}

/// `${dir.a.dir.b.file.page}` — multi-hop path through nested id-bearing dirs to a file child.
#[test]
fn multi_dir_path_to_file() {
    // Arrange
    let key = "dir.a.dir.b.file.page";

    // Act
    let result = parse_ref(key);

    // Assert
    match &result.head {
        RefHead::DirNs { id, tail } => {
            assert_eq!(id, "a");
            assert_eq!(
                tail,
                &vec![Hop::Dir("b".to_string()), Hop::File("page".to_string())]
            );
        }
        other => panic!("expected DirNs, got {other:?}"),
    }
}
