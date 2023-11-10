#![allow(unused)]
#![allow(clippy::all)]

use regex::{Matches, Regex, RegexBuilder};
use std::collections::HashMap;
use tracing::{error, info};
use tracing_subscriber;
use uuid::Uuid;
use yaml_rust::{YamlEmitter, YamlLoader};

#[derive(Debug)]
pub struct ParseField {
    name: String,
    data: String,
    modifiers: String,
    regex: String,
    format: String,
}

impl ParseField {
    pub fn new(parse_field: ParseField) -> Self {
        info!("MainData set()");

        let mut l_self = ParseField {
            name: "".to_string(),
            data: "".to_string(),
            modifiers: "".to_string(),
            regex: "".to_string(),
            format: "".to_string(),
        };

        match Regex::new(r"[^igm]") {
            //at this point only igm
            Ok(re_modifiers) => {
                l_self.name = parse_field.name;
                l_self.data = parse_field.data;
                l_self.modifiers = re_modifiers
                    .replace_all(&parse_field.modifiers, "")
                    .to_string();
                l_self.regex = parse_field.regex;
                if l_self.data.is_empty() && !l_self.regex.is_empty() {
                    l_self.data = l_self.regex.clone();
                }
                l_self.format = parse_field.format;
            }
            Err(_) => {}
        };

        l_self
    }
}

/**
 * ParseOption
 * @property {String} langu
 * @property {String} date_formats formate for 'date-fns parse' https://date-fns.org/v2.28.0/docs/parse
 * @property {Array} replace to replace the character in the text
 * @property {String} decimal_separator
 * @property {String} thousand_separator
 * @property {String} group_separator  i.e. ', ' if there are several groups of elements that should be linked together
 * @property {Boolean} remove_accents
 * @property {String} modifiers  i - case insensitive, g - global match, m - multiline, x - ignore space
 */
#[derive(Debug)]
pub struct ParseOption {
    pub langu: String,
    pub date_formats: String,
    pub replace: Vec<(String, String)>,
    pub required_fields: Vec<String>,
    pub decimal_separator: String,
    pub thousand_separator: String,
    pub group_separator: String,
    pub remove_accents: bool,
    pub modifiers: String, //i - case insensitive, g - global match, m - multiline, x - ignore space
    pub currency: String,
}

impl ParseOption {
    pub fn new(parse_option: ParseOption) -> Self {
        let mut l_self = ParseOption {
            langu: "en-UK".to_string(),
            date_formats: "dd.MM.yyyy".to_string(),
            replace: Vec::new(),
            required_fields: Vec::new(),
            decimal_separator: ".".to_string(),
            thousand_separator: ",".to_string(),
            group_separator: ",".to_string(),
            remove_accents: false,
            modifiers: "g".to_string(), //i - case insensitive, g - global match, m - multiline, x - ignore space
            currency: "EUR".to_string(),
        };

        if !parse_option.langu.is_empty() {
            l_self.langu = parse_option.langu;
        }
        if !parse_option.date_formats.is_empty() {
            l_self.date_formats = parse_option.date_formats;
        }
        for ele in &parse_option.replace {
            l_self.replace.push(ele.clone());
        }
        if !parse_option.decimal_separator.is_empty() {
            l_self.decimal_separator = parse_option.decimal_separator;
        }
        if !parse_option.thousand_separator.is_empty() {
            l_self.thousand_separator = parse_option.thousand_separator;
        }
        if !parse_option.group_separator.is_empty() {
            l_self.group_separator = parse_option.group_separator;
        }

        l_self.remove_accents = parse_option.remove_accents;

        if !parse_option.modifiers.is_empty() {
            l_self.modifiers = parse_option.modifiers;
        }

        l_self
    }

    pub fn new_by_lang(langu: &str) -> Self {
        if langu.starts_with("de") {
            ParseOption {
                langu: "de-DE".to_string(),
                date_formats: "dd.MM.yyyy".to_string(),
                replace: Vec::new(),
                required_fields: Vec::new(),
                decimal_separator: ",".to_string(),
                thousand_separator: ".".to_string(),
                group_separator: ",".to_string(),
                remove_accents: false,
                modifiers: "g".to_string(), //i - case insensitive, g - global match, m - multiline, x - ignore space
                currency: "EUR".to_string(),
            }
        } else {
            ParseOption {
                langu: "en-UK".to_string(),
                date_formats: "dd.MM.yyyy".to_string(),
                replace: Vec::new(),
                required_fields: Vec::new(),
                decimal_separator: ".".to_string(),
                thousand_separator: ",".to_string(),
                group_separator: ",".to_string(),
                remove_accents: false,
                modifiers: "g".to_string(), //i - case insensitive, g - global match, m - multiline, x - ignore space
                currency: "EUR".to_string(),
            }
        }
    }
}

///
/// ParseTemplate is the class for managing the templates for parsing a form.
/// @property {String} id uuid
/// @property {String} name the name for identification
/// @property {String} group the group classifies the template, e.g. invoices
/// @property {Array} test Array if String for testing
/// @property {ParseOption} Option are the general parameters of the parsing, e.g. format for date
/// @property {ParseField} fields Array of ParsField
/// @property {Array} protocol Array aof String
///
#[derive(Debug)]
pub struct ParseTemplate {
    pub id: String,
    pub name: String,
    pub group: String,
    pub test: Vec<String>,
    pub options: ParseOption,
    pub fields: Vec<ParseField>,
    pub protocol: Vec<String>,
    pub data: HashMap<String, String>,
    pub reg_ex: HashMap<String, Vec<String>>,
    pub parse_protocol: HashMap<String, Vec<String>>,
}

impl ParseTemplate {
    /**
     * # new
     */
    pub fn new(parse_template: ParseTemplate) -> Self {
        let mut l_self = parse_template;
        if l_self.id.is_empty() {
            l_self.id = Uuid::new_v4().to_string()
        }
        l_self
    }

    /**
     * # load_from_yaml
     *
     */

    pub fn load_from_yaml(docs: Vec<yaml_rust::Yaml>, langu: &str) -> Self {
        info!("load_from_yaml");

        let mut l_self = ParseTemplate {
            id: Uuid::new_v4().to_string(),
            name: "".to_string(),
            group: "".to_string(),
            test: Vec::new(),
            options: ParseOption::new_by_lang(langu),
            fields: Vec::new(),
            protocol: Vec::new(),
            data: HashMap::new(),
            reg_ex: HashMap::new(),
            parse_protocol: HashMap::new(),
        };
        //println!("\n{:#?}\n", l_self);
        //println!("\n{:#?}", &docs);

        l_self.protocol.push("start yaml loading".to_string());
        for ele in docs {
            if ele.as_hash().is_some() {
                for (key, value) in ele.as_hash().unwrap() {
                    //tag name
                    let key_string = match key.as_str() {
                        Some(data) => data,
                        _ => {
                            continue;
                        }
                    };

                    match key_string {
                        "name" | "issuer" => {
                            if value.as_str().is_some() {
                                l_self.name = value.as_str().unwrap().to_string();
                            } else {
                                l_self.protocol.push(
                                    "* error - wrong type for tag 'name' - we need string"
                                        .to_string(),
                                );
                            }
                        }

                        //tag group
                        "group" => {
                            if value.as_str().is_some() {
                                l_self.group = value.as_str().unwrap().to_string();
                            } else {
                                l_self.protocol.push(
                                    "* error - wrong type for tag 'group' - we need string"
                                        .to_string(),
                                );
                            }
                        }

                        //tag test
                        "test" | "keywords" => {
                            if value.as_str().is_some() {
                                l_self.test.push(value.as_str().unwrap().to_string());
                            } else if value.as_vec().is_some() {
                                for test_array in value.as_vec().unwrap() {
                                    if test_array.as_str().is_some() {
                                        l_self.test.push(test_array.as_str().unwrap().to_string());
                                    }
                                }
                            } else {
                                l_self.protocol.push("* error - wrong type for tag 'test' - we need string or array of string".to_string());
                            }
                        }

                        "options" => {
                            if value.as_hash().is_some() {
                                for (key2, value2) in value.as_hash().unwrap() {
                                    let key_string2 = match key2.as_str() {
                                        Some(data) => data,
                                        _ => {
                                            continue;
                                        }
                                    };

                                    match key_string2 {
                                        "langu" | "languages" => {
                                            if value2.as_str().is_some() {
                                                l_self.options.langu =
                                                    value2.as_str().unwrap().to_string();
                                            } else if value2.as_vec().is_some() {
                                                for langu_array in value2.as_vec().unwrap() {
                                                    if langu_array.as_str().is_some() {
                                                        l_self.options.langu = langu_array
                                                            .as_str()
                                                            .unwrap()
                                                            .to_string();
                                                    }
                                                }
                                            } else {
                                                l_self.protocol.push(
                                            "* error - wrong type for tag 'langu' or 'languages' - we need string"
                                                .to_string(),
                                        );
                                            }
                                        }

                                        "decimal_separator" => {
                                            if value2.as_str().is_some() {
                                                l_self.options.decimal_separator =
                                                    value2.as_str().unwrap().to_string();
                                            } else {
                                                l_self.protocol.push(
                                            "* error - wrong type for tag 'decimal_separator' - we need string".to_string(),
                                        );
                                            }
                                        }

                                        "thousand_separator" => {
                                            if value2.as_str().is_some() {
                                                l_self.options.thousand_separator =
                                                    value2.as_str().unwrap().to_string();
                                            } else {
                                                l_self.protocol.push(
                                            "* error - wrong type for tag 'thousand_separator' - we need string".to_string(),
                                        );
                                            }
                                        }

                                        "group_separator" => {
                                            if value2.as_str().is_some() {
                                                l_self.options.group_separator =
                                                    value2.as_str().unwrap().to_string();
                                            } else {
                                                l_self.protocol.push(
                                            "* error - wrong type for tag 'group_separator' - we need string".to_string(),
                                        );
                                            }
                                        }

                                        "remove_accents" => {
                                            if value2.as_bool().is_some() {
                                                l_self.options.remove_accents =
                                                    value2.as_bool().unwrap();
                                            } else {
                                                l_self.protocol.push(
                                            "* error - wrong type for tag 'remove_accents' - we need boolean".to_string(),);
                                            }
                                        }

                                        "remove_whitespace" => {
                                            if value2.as_bool().is_some() {
                                                if value2.as_bool().unwrap() == true {
                                                    l_self.options.modifiers.push_str("x");
                                                }
                                            //remove_whitespace
                                            } else {
                                                l_self.protocol.push(
                                            "* error - wrong type for tag 'remove_whitespace' - we need boolean".to_string(),);
                                            }
                                        }

                                        "lowercase" => {
                                            if value2.as_bool().is_some() {
                                                if value2.as_bool().unwrap() == true {
                                                    l_self.options.modifiers.push_str("i");
                                                }
                                            //caseinsensetive
                                            } else {
                                                l_self.protocol.push(
                                            "* error - wrong type for tag 'lowercase' - we need boolean".to_string(),);
                                            }
                                        }

                                        "modifiers" => {
                                            if value2.as_str().is_some() {
                                                l_self.options.modifiers =
                                                    value2.as_str().unwrap().to_string();
                                            } else {
                                                l_self.protocol.push(
                                            "* error - wrong type for tag 'modifiers' - we need string"
                                                .to_string(),
                                        );
                                            }
                                        }

                                        "currency" => {
                                            if value2.as_str().is_some() {
                                                l_self.options.currency =
                                                    value2.as_str().unwrap().to_string();
                                            } else {
                                                l_self.protocol.push(
                                            "* error - wrong type for tag 'currency' - we need string"
                                                .to_string(),
                                        );
                                            }
                                        }

                                        "date_formats" => {
                                            if value2.as_str().is_some() {
                                                l_self.options.date_formats =
                                                    value2.as_str().unwrap().to_string();
                                            } else if value2.as_vec().is_some() {
                                                for langu_array in value2.as_vec().unwrap() {
                                                    if langu_array.as_str().is_some() {
                                                        l_self.options.date_formats = langu_array
                                                            .as_str()
                                                            .unwrap()
                                                            .to_string();
                                                    }
                                                }
                                            } else {
                                                l_self.protocol.push(
                                            "* error - wrong type for tag 'date_formats' - we need string".to_string(),
                                        );
                                            }
                                        }

                                        "replace" => {
                                            if value2.as_vec().is_some() {
                                                for replace_array in value2.as_vec().unwrap() {
                                                    if replace_array.as_vec().is_some() {
                                                        if replace_array.as_vec().is_some()
                                                            && replace_array.as_vec().unwrap().len()
                                                                > 1
                                                        {
                                                            l_self.options.replace.push((
                                                                replace_array.as_vec().unwrap()[0]
                                                                    .as_str()
                                                                    .unwrap()
                                                                    .to_string(),
                                                                replace_array.as_vec().unwrap()[1]
                                                                    .as_str()
                                                                    .unwrap()
                                                                    .to_string(),
                                                            ))
                                                        } else {
                                                            l_self.protocol.push("* error - wrong type for tag 'replace' - we need string or array of string".to_string());
                                                        }
                                                    } else {
                                                        l_self.protocol.push("* error - wrong type for tag 'replace' - we need string or array of string".to_string());
                                                    }
                                                }
                                            } else {
                                                l_self.protocol.push("* error - wrong type for tag 'replace' - we need string or array of string".to_string());
                                            }
                                        }

                                        "required_fields" => {
                                            if value2.as_vec().is_some() {
                                                for required_fields in value2.as_vec().unwrap() {
                                                    if required_fields.as_str().is_some() {
                                                        l_self.options.required_fields.push(
                                                            required_fields
                                                                .as_str()
                                                                .unwrap()
                                                                .to_string(),
                                                        );
                                                    } else {
                                                        l_self.protocol.push("* error - wrong type for tag 'required_fields' - we need string".to_string());
                                                    }
                                                }
                                            } else {
                                                l_self.protocol.push("* error - wrong type for tag 'required_fields' - we need string or array of string".to_string());
                                            }
                                        }
                                        _ => {
                                            l_self.protocol.push(
                                                format!("* error - wrong key '{}'", key_string2)
                                                    .to_string(),
                                            );
                                        }
                                    };
                                }
                            } else {
                                l_self.protocol.push(
                                    "* error - wrong type for tag 'options' - we need hash"
                                        .to_string(),
                                );
                            };
                        }

                        "fields" => {
                            if value.as_hash().is_some() {
                                for (key2, value2) in value.as_hash().unwrap() {
                                    let key_string2 = match key2.as_str() {
                                        Some(data) => data,
                                        _ => {
                                            continue;
                                        }
                                    };

                                    if value2.as_str().is_some() {
                                        l_self.fields.push(ParseField {
                                            name: key_string2.to_string(),
                                            data: "".to_string(),
                                            modifiers: "".to_string(),
                                            regex: value2.as_str().unwrap().to_string(),
                                            format: "".to_string(),
                                        })
                                    } else if value2.as_hash().is_some() {
                                        l_self.fields.push(ParseField {
                                            name: key_string2.to_string(),
                                            data: "".to_string(),
                                            modifiers: "".to_string(),
                                            regex: "".to_string(),
                                            format: "".to_string(),
                                        });
                                        let index = match l_self
                                            .fields
                                            .iter()
                                            .position(|r| r.name == key_string2)
                                        {
                                            Some(index) => index,
                                            _ => {
                                                continue;
                                            }
                                        };
                                        for (key3, value3) in value2.as_hash().unwrap() {
                                            if key3.as_str().is_some()
                                                && key3.as_str().unwrap() == "regex"
                                                && value3.as_str().is_some()
                                            {
                                                l_self.fields[index].regex =
                                                    value3.as_str().unwrap().to_string();
                                            } else if key3.as_str().is_some()
                                                && key3.as_str().unwrap() == "modifiers"
                                                && value3.as_str().is_some()
                                            {
                                                l_self.fields[index].modifiers =
                                                    value3.as_str().unwrap().to_string();
                                            } else {
                                                l_self.protocol.push(
                                                "* error - wrong tag 'fields' - we need 'regex' or 'modifiers'".to_string(),
                                            );
                                            }
                                        }
                                    } else {
                                        l_self.protocol.push(
                                        "* error - wrong type for tag 'fields' - we need string or array".to_string(),
                                    );
                                    }
                                }
                            } else {
                                l_self.protocol.push(
                                "* error - wrong type for tag 'fields' - we need hash of tuple or array".to_string(),
                            );
                            }
                        }
                        _ => {
                            l_self
                                .protocol
                                .push(format!("* error - wrong key '{}'", key_string).to_string());
                        }
                    };
                }
            } else {
                l_self
                    .protocol
                    .push("* error - no hash tag found".to_string());
            }
        }
        l_self.protocol.push("stop yaml loading".to_string());

        //println!("\n{:#?}", &l_self);

        l_self
    }

    ///
    /// check error occurred; the protocol string must start with ERROR<br>
    /// @returns true - error occurred
    ///
    pub fn error_occurred(&self) -> bool {
        for ele in &self.protocol {
            if ele.to_uppercase().starts_with("ERROR") || ele.to_uppercase().starts_with("* ERROR")
            {
                return true;
            }
        }

        false
    }

    ///
    /// Clear protocol
    ///
    pub fn clear_protocol(&mut self) {
        info!("clear_protocol");

        self.protocol = Vec::new();
    }
    /**
     * Checks if the template matches the "test".
     * all conditions must be met
     * @param {String} iText the text to parse
     * @returns {Boolean} true - the template is valid
     */
    pub fn perform_test(&self, i_text: &String) -> bool {
        info!("perform_test name {}", &self.name);

        if self.test.len() == 0 {
            return false;
        }

        let mut case_insensitive = false;
        let mut ignore_whitespace = false;
        let mut result = false;
        for ele in &self.test {
            let re = Regex::new("\\/[xig]*$").unwrap();
            let mut l_extra_ele = re.replace_all(&ele, "").to_string();
            let re = Regex::new("\\/[xg]*i[xg]*$").unwrap();
            if re.is_match(&ele) {
                case_insensitive = true;
            }
            let re = Regex::new("\\/[ig]*x[ig]*$").unwrap();
            if re.is_match(&ele) {
                //x - ignore space
                ignore_whitespace = true;

                let re = Regex::new("[ ]*").unwrap();
                l_extra_ele = re.replace_all(&l_extra_ele, "").to_string();
                if l_extra_ele.is_empty() {
                    continue; //next item if regex is empty
                }
                let re = match RegexBuilder::new(&l_extra_ele)
                    .case_insensitive(case_insensitive)
                    .ignore_whitespace(ignore_whitespace)
                    .build()
                {
                    Ok(l_re) => l_re,
                    Err(_) => {
                        continue; //next item if regex is wrong
                    }
                };
                if !re.is_match(&i_text.replace(" ", "")) {
                    return false;
                }
            } else {
                let re = match RegexBuilder::new(&l_extra_ele)
                    .case_insensitive(case_insensitive)
                    .ignore_whitespace(ignore_whitespace)
                    .build()
                {
                    Ok(l_re) => l_re,
                    Err(_) => {
                        continue; //next item if regex is wrong
                    }
                };
                if !re.is_match(&i_text) {
                    return false;
                }
            }
            result = true;
        }

        return result;
    }

    ///
    /// extract all fields of the ParseTemplate from the text
    /// @i_text {String} iText is the raw text
    /// @returns {Object} {data, regex, protocol, error} the parsed fields are in data as a json object
    ///
    pub fn parse_data(&mut self, i_text: &String) {
        info!("parse_data");

        let mut e_text: String = i_text.clone();

        if e_text.is_empty() {
            self.protocol.push("error - text is empty".to_string());
            return;
        }

        //Replace all substitutions
        for (item_from, item_to) in &self.options.replace {
            if item_from.is_empty() {
                continue;
            }
            match Regex::new(item_from) {
                Ok(re) => e_text = re.replace_all(&e_text, item_to).to_string(),
                Err(_) => {}
            };
        }

        //Replace all remove_accents
        if self.options.remove_accents {
            //use unicode_normalization::char::compose;
            use unicode_normalization::UnicodeNormalization;
            e_text = e_text.nfd().collect::<String>();
            match Regex::new("[\\u0300-\\u036f]") {
                Ok(re) => {
                    e_text = re.replace_all(&e_text, "").to_string();
                }
                Err(_) => {}
            }
        }

        let e_text_no_whitespace = e_text.replace(" ", "");

        for field in &self.fields {
            let real_name = field.name.replace("static_", "");
            //println!("realname: {}", &real_name);

            if real_name.is_empty() {
                self.protocol
                    .push("error - field name is empty".to_string());
                continue;
            }

            self.data.insert(real_name.clone(), "".to_string());

            if field.name.starts_with("static_") {
                //static -> regex is data value
                self.data.insert(real_name, field.regex.clone());
            } else {
                //regex
                self.reg_ex.insert(real_name.clone(), Vec::new());
                self.data.insert(real_name.clone(), "".to_string());

                let mut re_bu: RegexBuilder = RegexBuilder::new(&field.regex);
                let mut case_insensitive = false;
                let mut ignore_whitespace = false;
                let mut multi_line = false;

                //i - case insensitive, g - global match, m - multiline, x - ignore space
                if self.options.modifiers.contains("i") || field.modifiers.contains("i") {
                    //case_insensitive
                    case_insensitive = true;
                }
                if self.options.modifiers.contains("x") || field.modifiers.contains("x") {
                    //ignore_whitespace
                    ignore_whitespace = true;
                }
                if self.options.modifiers.contains("m") || field.modifiers.contains("m") {
                    multi_line = true;
                }

                let re = RegexBuilder::new(&field.regex)
                    .case_insensitive(case_insensitive)
                    .ignore_whitespace(ignore_whitespace)
                    .multi_line(multi_line)
                    .build()
                    .unwrap();

                let l_captures_iter = match ignore_whitespace {
                    true => re.captures_iter(&e_text_no_whitespace),
                    false => re.captures_iter(&e_text),
                };

                let mut count = 0;
                for l_capture in l_captures_iter {
                    count += 1;

                    let mut l_cap_string = "".to_string();
                    for i in 1..10 {
                        let l_sub_str = l_capture.get(i).map_or("", |m| m.as_str());
                        if !l_sub_str.is_empty() {
                            if !l_cap_string.is_empty() {
                                l_cap_string.push_str(" "); //separator
                            }
                            l_cap_string.push_str(l_sub_str)
                        };
                    }

                    if l_cap_string.is_empty() {
                        // if no groups found
                        l_cap_string = l_capture.get(0).map_or("", |m| m.as_str()).to_string();
                    }

                    self.reg_ex
                        .get_mut(&real_name)
                        .unwrap()
                        .push(l_cap_string.to_owned().clone());

                    if !l_cap_string.is_empty() && field.name.ends_with("_float") {
                        if !self.options.thousand_separator.is_empty()
                            && self.options.thousand_separator != self.options.decimal_separator
                        {
                            l_cap_string =
                                l_cap_string.replace(&self.options.thousand_separator, "");
                        }
                        if !self.options.decimal_separator.is_empty() {
                            l_cap_string =
                                l_cap_string.replace(&self.options.decimal_separator, ".");
                        }
                    }

                    if !l_cap_string.is_empty() && field.name.ends_with("_date") {
                        if !self.options.date_formats.is_empty() {
                            use chrono::NaiveDate;
                            let parse_from_str = NaiveDate::parse_from_str;
                            match parse_from_str(&l_cap_string, &self.options.date_formats) {
                                Ok(native_data) => {
                                    l_cap_string = native_data.to_string();
                                    //info!("{}", native_data);
                                }
                                _ => {}
                            };
                        }
                    }

                    let mut l_data_string = self
                        .data
                        .get(&real_name)
                        .unwrap_or(&"".to_string())
                        .to_owned();
                    if !l_data_string.is_empty() {
                        l_data_string.push_str(", ");
                    }
                    l_data_string.push_str(&l_cap_string);

                    if field.modifiers.contains("g") {
                        self.data.insert(real_name.clone(), l_data_string);
                    } else if count == 1 {
                        self.data.insert(real_name.clone(), l_data_string);
                    }
                }
            }
        }
    }
}

pub fn test_parse() {
    println!("test_parse");

    //read and convert yaml-file to rust struct
    let l_yaml_file = match std::fs::read_to_string("/Users/jan/nodejs/formparser/test/test.yaml") {
        Ok(data) => data,
        Err(err) => panic!("{}", err),
    };

    let l_yaml = YamlLoader::load_from_str(&l_yaml_file).unwrap();
    //println!("\n{:?}\n", l_yaml);

    let mut my_template = ParseTemplate::load_from_yaml(l_yaml, "de-DE");

    let l_text = match std::fs::read_to_string(
        "/Users/jan/nodejs/formparser/test/ad89bea8-bd96-4cfd-942d-8d1e06cd9271000.jpg.txt",
    ) {
        Ok(data) => data,
        Err(err) => panic!("{}", err),
    };

    //Test a text for validity (test)to the ParseTemplate
    let l_valide = my_template.perform_test(&l_text);
    println!("perform_test : {}", l_valide);

    if (l_valide) {
        //extract all fields of the ParseTemplate from the text
        my_template.parse_data(&l_text);
        //console.log(JSON.stringify({ lData, lRegEx, lProtocol, lError }, null, 2))
    }

    println!("\n{:#?}˜n", my_template);
}
