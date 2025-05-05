use std::{collections::HashMap, str::FromStr};

/// Parse a string to another type T.
///
/// T must implement trait `std::str::FromStr`
///
/// # Example
/// ```
///  use scylla_pg_monitor::env::parse_from_string;
/// let x = parse_from_string::<u32>("20".to_string());
/// assert_eq!(x, Ok(20));
/// ```
///
/// The above example will parse the string `"20"` to return 20 as u32.
pub fn parse_from_string<T: std::str::FromStr>(value: String) -> Result<T, String>
where
    <T as FromStr>::Err: std::fmt::Debug,
{
    Ok(value.trim().parse::<T>().expect("Failed to parse!!"))
}

/// Gets the value for the key passed to `get_env_var_value`
///
/// # Example
/// ```
/// use std::env;
/// env::set_var("TEST", "valueA");
/// use scylla_pg_monitor::env::get_env_var_value;
/// let x = get_env_var_value("TEST");
/// ```
///
/// The above example will return the value for key `TEST`
pub fn get_env_var_value(key: &str) -> Result<String, String> {
    Ok(std::env::var(key).unwrap_or_else(|_| panic!("{} environment variable is not defined", key)))
}

/// Returns all the env variables matching a prefix
///
/// # Example
/// ```
/// # use scylla_pg_monitor::env::get_env_vars_with_prefix;
/// let x = get_env_vars_with_prefix("TEST");
/// ```
///
/// The above example will return all the env variables that is prefixed with `TEST`
pub fn get_env_vars_with_prefix(prefix: &str) -> Option<HashMap<String, String>> {
    let matching_props = std::env::vars().filter(|x| x.0.contains(prefix)).fold(HashMap::new(), |mut acc, x| {
        acc.insert(x.0, x.1);
        acc
    });
    Some(matching_props)
}

/// Read environment variable using a key.
///
/// - When only (key) is passed, returns the value as a `String`.
///
/// ## Example
/// ```
/// use scylla_pg_monitor::env_var;
/// use std::env;
/// env::set_var("keyA", "valueA");
/// let x:String = env_var!("keyA"); // returns `valueA` as a String.
/// assert_eq!(x, "valueA".to_string());
/// ```
///
/// - When the `key` and value return `type` is passed, the environment variable is
/// read for the key and the value is parsed into the `type` passed as argument.
///
/// ## Example
/// ```
/// use scylla_pg_monitor::env_var;
/// use std::env;
/// env::set_var("keyA", "20");
/// let x:String = env_var!("keyA"); // returns `valueA` as a String.
/// let x:u32 = env_var!("keyA", u32); // returns `20` as a String.
/// assert_eq!(x, 20);
/// ```
///
/// - Special scenario to convert the string value to Vector.
/// When the `key` and value return `type` is passed as `Vec<type>`
///     - the environment variable is read for the key.
///     - the string value returned is split on `,` to create a Vec.
///     - each value of the vec is parsed into the `type` passed as argument.
///
/// ## Example
/// ```
/// use scylla_pg_monitor::env_var;
/// use std::env;
/// env::set_var("keyA", "testA, testB");
/// let x:Vec<String> = env_var!("keyA", Vec<String>);
/// assert_eq!(x[0], "testA".to_string());
/// ```
#[macro_export]
macro_rules! env_var {
    ($key: expr) => {
        std::env::var($key).unwrap_or_else(|_| panic!("{} environment variable is not defined", $key))
    };
    ($key: expr, Vec<$type: ty>) => {{
        let value_string = env_var!($key);
        let value_vec = value_string.split(',').map(|v| {
            v.trim()
                .parse::<$type>()
                .unwrap_or_else(|_| panic!("error parsing \"{value_string}\" String -> Vec<{}>", stringify!($type)))
        });
        value_vec.collect::<Vec<$type>>()
    }};
    ($key: expr, $type: ty) => {{
        let value_string = env_var!($key);
        value_string
            .parse::<$type>()
            .unwrap_or_else(|_| panic!("error parsing \"{value_string}\" String -> {}", stringify!($type)))
    }};
}

/// Read environment variable using a key and default value.
///
/// - When only (key, default_value) is passed, returns the value or the default value.
///
/// - When the `key`, value return `type` as an `Option` and `default_value` is passed, the environment variable is
/// read for the key and the value is parsed into the `type` passed as argument. If the value is not found,
/// then the default value is assigned.
///
/// - When the `key`, value return `type` and `default_value` is passed, the environment variable is
/// read for the key and the value is parsed into the `type` passed as argument. If the value is not found,
/// then the default value is assigned.
///
#[macro_export]
macro_rules! env_var_with_defaults {
    ($key: expr, Option::<$type: ty>, $default: expr ) => {{
        let default_value: $type = $default;
        let value_string = std::env::var($key);
        let res = if let Ok(v) = value_string {
            Some(v.parse::<$type>().unwrap())
        } else {
            Some(default_value)
        };
        res
    }};
    ($key: expr, Option::<$type: ty>) => {{
        let value_string = std::env::var($key);
        let res = if let Ok(v) = value_string { Some(v.parse::<$type>().unwrap()) } else { None };
        res
    }};
    ($key: expr, $default: expr) => {
        std::env::var($key).unwrap_or_else(|_| $default)
    };
    ($key: expr, $type: ty, $default: expr ) => {{
        let value_string = std::env::var($key);
        let res = if let Ok(v) = value_string { v.parse::<$type>().unwrap() } else { $default };
        res
    }};
}

#[cfg(test)]
mod tests {
    use std::env;

    use serial_test::serial;

    fn set_env_var(key: &str, value: &str) {
        env::set_var(key, value)
    }

    fn unset_env_var(key: &str) {
        env::remove_var(key)
    }

    #[test]
    #[serial]
    fn test_env_var_macro_get_value_successfully_for_key() {
        // When only the key is passed a value as String is returned.
        set_env_var("keyA", "valueA");

        let val = env_var!("keyA");
        assert_eq!(val, "valueA".to_owned());

        // When the key and value type is passed, the value is returned
        // parsed as the type.
        set_env_var("keyA", "true");
        let val = env_var!("keyA", bool);
        assert!(val);

        // When the key and value type is passed as a Vec<type>, the value is returned
        // parsed as Vec<type>.
        set_env_var("keyA", "12, 20, 33");
        let val = env_var!("keyA", Vec<u32>);
        assert_eq!(val.len(), 3);

        let first_val = *val.first().unwrap();
        assert_eq!(first_val, 12);

        unset_env_var("keyA");
    }
    #[test]
    #[serial]
    #[should_panic(expected = "keyB environment variable is not defined")]
    fn test_env_var_macro_when_key_value_not_found() {
        // When only the key is passed a value as String is returned.
        set_env_var("keyAE1", "valueA");

        let _val = env_var!("keyB");

        unset_env_var("keyAE1");
    }
    #[test]
    #[serial]
    #[should_panic(expected = "error parsing \"valueA\" String -> u32")]
    fn test_env_var_macro_when_parsing_fails() {
        // When only the key is passed a value as String is returned.
        set_env_var("keyAE2", "valueA");

        let _val = env_var!("keyAE2", u32);

        unset_env_var("keyAE2");
    }
    #[test]
    #[serial]
    #[should_panic(expected = "error parsing \"1, 2 ,valueA\" String -> Vec<u32>")]
    fn test_env_var_macro_when_parsing_fails_for_vector() {
        // When only the key is passed a value as String is returned.
        set_env_var("keyAE3", "1, 2 ,valueA");

        let _val = env_var!("keyAE3", Vec<u32>);

        unset_env_var("keyAE3");
    }

    #[test]
    #[serial]
    fn test_env_var_with_default_macro_get_value_successfully_for_key() {
        let val = env_var_with_defaults!("keyA", "test_string".to_owned());
        assert_eq!(val, "test_string".to_owned());

        set_env_var("keyA", "30");
        let val = env_var_with_defaults!("keyA", Option::<u64>, 10);

        assert_eq!(val, Some(30));

        unset_env_var("keyA");

        let val = env_var_with_defaults!("keyA", Option::<u64>);
        assert!(val.is_none());

        set_env_var("keyA", "30");
        let val = env_var_with_defaults!("keyA", u64, 10);
        assert_eq!(val, 30);

        unset_env_var("keyA");

        // let res: Option<u64> = if value_string.is_none() {
        //     Some(10)
        // } else {
        //     value_string.map(|m| m.to_string().parse::<u64>().unwrap())
        // };

        // // When the key and value type is passed, the value is returned
        // // parsed as the type.
        // set_env_var("keyA", "true");
        // let val = env_var!("keyA", bool);
        // assert!(val);

        // // When the key and value type is passed as a Vec<type>, the value is returned
        // // parsed as Vec<type>.
        // set_env_var("keyA", "12, 20, 33");
        // let val = env_var!("keyA", Vec<u32>);
        // assert_eq!(val.len(), 3);

        // let first_val = *val.first().unwrap();
        // assert_eq!(first_val, 12);

        unset_env_var("keyA");
    }
}
