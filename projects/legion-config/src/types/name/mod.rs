use super::*;

// `name`
// `@user/name`
// `@user/name-path`
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize)]
pub struct PackageName {
    user: String,
    name: String,
}

impl PackageName {
    pub fn new(user: &str, name: &str) -> Result<Self, SyntaxError> {
        let mut out = PackageName::default();
        out.set_user(user)?;
        out.set_name(name)?;
        Ok(out)
    }

    pub fn is_empty(&self) -> bool {
        self.name.is_empty()
    }

    pub fn get_user(&self) -> &str {
        &self.user
    }
    pub fn set_user(&mut self, user: &str) -> Result<(), SyntaxError> {
        let new = regularize(user)?;
        if new.starts_with('-') || new.ends_with('-') {
            Err(SyntaxError::new("package user cannot start or end with `-`"))?
        }
        if new.contains("--") {
            Err(SyntaxError::new("package user cannot contain `--`"))?
        }
        self.user = new;
        Ok(())
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn set_name(&mut self, name: &str) -> Result<(), SyntaxError> {
        let new = regularize(name)?;
        if new.is_empty() {
            Err(SyntaxError::new("package name cannot be empty"))?
        }
        if new.starts_with(|c: char| c.is_numeric()) {
            Err(SyntaxError::new("package name cannot start with a number"))?
        }
        if new.starts_with('-') || new.ends_with('-') {
            Err(SyntaxError::new("package name cannot start or end with `-`"))?
        }
        if new.contains("--") {
            Err(SyntaxError::new("package name cannot contain `--`"))?
        }
        self.name = new;
        Ok(())
    }
}

impl Default for PackageName {
    fn default() -> Self {
        Self { user: "".to_string(), name: "".to_string() }
    }
}

impl Display for PackageName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.user.is_empty() { f.write_str(&self.name) } else { f.write_str(&format!("@{}/{}", self.user, self.name)) }
    }
}

impl FromStr for PackageName {
    type Err = SyntaxError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut user = "";
        let mut name = "";
        if input.trim().starts_with('@') {
            match input.split_once('/') {
                Some(s) => {
                    user = s.0.trim_start_matches('@');
                    name = s.1;
                }
                None => Err(SyntaxError::new("package name must be in the format of @user/name"))?,
            }
        }
        else {
            name = input;
        }
        PackageName::new(user, name)
    }
}

fn regularize(input: &str) -> Result<String, SyntaxError> {
    let mut out = String::with_capacity(input.len());
    for char in input.chars() {
        match char {
            'A'..='Z' => out.push(char.to_ascii_lowercase()),
            'a'..='z' => out.push(char),
            '0'..='9' => out.push(char),
            '-' | '_' | ' ' => out.push('-'),
            _ => Err(SyntaxError::new(format!("invalid character `{}` in package name", char)))?,
        }
    }
    Ok(out)
}

bind_writer!(PackageNameWriter, PackageName);

impl<'i, 'de> Visitor<'de> for PackageNameWriter<'i> {
    type Value = ();

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("Expect a string or a `ValorPackageName` object.")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match PackageName::from_str(v) {
            Ok(o) => *self.ptr = o,
            Err(e) => Err(E::custom(e))?,
        }
        Ok(())
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        while let Some((key, value)) = map.next_entry::<String, String>()? {
            match key.as_str() {
                "user" => self.ptr.user = value,
                "name" => self.ptr.name = value,
                _ => {}
            }
        }
        Ok(())
    }
}
