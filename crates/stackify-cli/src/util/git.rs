pub enum TargetType {
    Tag,
    Branch,
    Commit,
}

pub struct GitTarget {
    pub target_type: TargetType,
    pub target: String,
}

impl GitTarget {
    pub fn parse<T: AsRef<str>>(s: T) -> Option<GitTarget> {
        let s = s.as_ref();
        let split = s.split(":").collect::<Vec<_>>();
        if split.len() < 2 {
            return None;
        }
        let target_type = match split[0] {
            "tag" => TargetType::Tag,
            "branch" => TargetType::Branch,
            "commit" => TargetType::Commit,
            _ => return None,
        };
        Some(GitTarget {
            target_type,
            target: split[1].to_string(),
        })
    }

    pub fn parse_opt<T: AsRef<str>>(s: Option<T>) -> Option<GitTarget> {
        if let Some(s) = s {
            Self::parse(s)
        } else {
            None
        }
    }
}
