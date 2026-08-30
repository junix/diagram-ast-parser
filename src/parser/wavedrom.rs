use crate::{
    ast::wavedrom::{
        WaveDromDocument, WaveGroup, WaveHeaderFooter, WaveLane, WaveRegisterDiagram,
        WaveRegisterField, WaveSignalItem, WaveTimingDiagram,
    },
    Format, ParseError, ParseResult, Span,
};
use serde_json::{Map, Value};

pub(crate) fn parse(source: &str) -> ParseResult<WaveDromDocument> {
    let value: Value = json5::from_str(source).map_err(|error| {
        ParseError::new(
            Format::WaveDrom,
            format!("invalid WaveJSON/JSON5: {error}"),
            None,
            source,
        )
    })?;

    let mut root = value.as_object().cloned().ok_or_else(|| {
        ParseError::new(
            Format::WaveDrom,
            "WaveDrom input must be a JSON5 object",
            None,
            source,
        )
    })?;

    let signal = root.remove("signal");
    let edges = root.remove("edge");
    let head = root.remove("head");
    let foot = root.remove("foot");
    let config = root.remove("config");
    let register_fields = root.remove("reg");

    let timing = if signal.is_some() || edges.is_some() || head.is_some() || foot.is_some() {
        Some(WaveTimingDiagram {
            signal: parse_signal_array(signal.as_ref(), source)?,
            edges: parse_string_array(edges.as_ref(), "edge", source)?,
            head: parse_header_footer(head.as_ref(), "head", source)?,
            foot: parse_header_footer(foot.as_ref(), "foot", source)?,
            config: config.clone(),
        })
    } else {
        None
    };

    let register = register_fields
        .as_ref()
        .map(|value| parse_register(value, config.clone(), source))
        .transpose()?;

    if timing.is_none() && register.is_none() {
        return Err(ParseError::new(
            Format::WaveDrom,
            "expected at least one WaveDrom `signal`, `edge`, `head`, `foot`, or `reg` field",
            None,
            source,
        ));
    }

    Ok(WaveDromDocument {
        span: Span::new(0, source.len()),
        timing,
        register,
        extra: root,
    })
}

fn parse_signal_array(value: Option<&Value>, source: &str) -> ParseResult<Vec<WaveSignalItem>> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let array = value.as_array().ok_or_else(|| {
        ParseError::new(Format::WaveDrom, "`signal` must be an array", None, source)
    })?;
    array
        .iter()
        .map(|item| parse_signal_item(item, source))
        .collect()
}

fn parse_signal_item(value: &Value, source: &str) -> ParseResult<WaveSignalItem> {
    if let Some(object) = value.as_object() {
        return Ok(WaveSignalItem::Lane(parse_lane(object.clone(), source)?));
    }

    let array = value.as_array().ok_or_else(|| {
        ParseError::new(
            Format::WaveDrom,
            "a signal item must be a lane object or a group array",
            None,
            source,
        )
    })?;
    let (label, rest) = array.split_first().ok_or_else(|| {
        ParseError::new(
            Format::WaveDrom,
            "a signal group cannot be empty",
            None,
            source,
        )
    })?;
    let label = label.as_str().ok_or_else(|| {
        ParseError::new(
            Format::WaveDrom,
            "the first item in a signal group must be its string label",
            None,
            source,
        )
    })?;
    let items = rest
        .iter()
        .map(|item| parse_signal_item(item, source))
        .collect::<ParseResult<Vec<_>>>()?;
    Ok(WaveSignalItem::Group(WaveGroup {
        label: label.to_owned(),
        items,
    }))
}

fn parse_lane(mut object: Map<String, Value>, source: &str) -> ParseResult<WaveLane> {
    let name = take_optional_string(&mut object, "name", source)?;
    let wave = take_optional_string(&mut object, "wave", source)?;
    let node = take_optional_string(&mut object, "node", source)?;
    let phase = take_optional_number(&mut object, "phase", source)?;
    let period = take_optional_number(&mut object, "period", source)?;
    let data = match object.remove("data") {
        None => Vec::new(),
        Some(Value::String(value)) => value.split_whitespace().map(str::to_owned).collect(),
        Some(Value::Array(values)) => values
            .iter()
            .map(value_to_string)
            .collect::<ParseResult<Vec<_>>>()?,
        Some(_) => {
            return Err(ParseError::new(
                Format::WaveDrom,
                "lane `data` must be a string or an array of scalar values",
                None,
                source,
            ));
        }
    };

    Ok(WaveLane {
        name,
        wave,
        data,
        node,
        phase,
        period,
        extra: object,
    })
}

fn parse_header_footer(
    value: Option<&Value>,
    field: &str,
    source: &str,
) -> ParseResult<Option<WaveHeaderFooter>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let mut object = value.as_object().cloned().ok_or_else(|| {
        ParseError::new(
            Format::WaveDrom,
            format!("`{field}` must be an object"),
            None,
            source,
        )
    })?;
    Ok(Some(WaveHeaderFooter {
        text: take_optional_string(&mut object, "text", source)?,
        tick: take_optional_integer(&mut object, "tick", source)?,
        tock: take_optional_integer(&mut object, "tock", source)?,
        every: take_optional_integer(&mut object, "every", source)?,
        extra: object,
    }))
}

fn parse_register(
    value: &Value,
    config: Option<Value>,
    source: &str,
) -> ParseResult<WaveRegisterDiagram> {
    let fields = value
        .as_array()
        .ok_or_else(|| ParseError::new(Format::WaveDrom, "`reg` must be an array", None, source))?;
    let fields = fields
        .iter()
        .map(|field| {
            let mut object = field.as_object().cloned().ok_or_else(|| {
                ParseError::new(
                    Format::WaveDrom,
                    "each `reg` field must be an object",
                    None,
                    source,
                )
            })?;
            let bits = match object.remove("bits") {
                None => None,
                Some(Value::Number(value)) => Some(value.as_u64().ok_or_else(|| {
                    ParseError::new(
                        Format::WaveDrom,
                        "register field `bits` must be an unsigned integer",
                        None,
                        source,
                    )
                })?),
                Some(_) => {
                    return Err(ParseError::new(
                        Format::WaveDrom,
                        "register field `bits` must be an unsigned integer",
                        None,
                        source,
                    ));
                }
            };
            let name = take_optional_string(&mut object, "name", source)?;
            let attr = object.remove("attr");
            let field_type = object.remove("type");
            Ok(WaveRegisterField {
                bits,
                name,
                attr,
                field_type,
                extra: object,
            })
        })
        .collect::<ParseResult<Vec<_>>>()?;
    Ok(WaveRegisterDiagram { fields, config })
}

fn parse_string_array(
    value: Option<&Value>,
    field: &str,
    source: &str,
) -> ParseResult<Vec<String>> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let array = value.as_array().ok_or_else(|| {
        ParseError::new(
            Format::WaveDrom,
            format!("`{field}` must be an array"),
            None,
            source,
        )
    })?;
    array.iter().map(value_to_string).collect()
}

fn value_to_string(value: &Value) -> ParseResult<String> {
    match value {
        Value::String(value) => Ok(value.clone()),
        Value::Number(value) => Ok(value.to_string()),
        Value::Bool(value) => Ok(value.to_string()),
        Value::Null => Ok("null".to_owned()),
        Value::Array(_) | Value::Object(_) => Err(ParseError {
            format: Format::WaveDrom,
            message: "expected a scalar value".to_owned(),
            span: None,
            line: 1,
            column: 1,
        }),
    }
}

fn take_optional_string(
    object: &mut Map<String, Value>,
    key: &str,
    source: &str,
) -> ParseResult<Option<String>> {
    match object.remove(key) {
        None => Ok(None),
        Some(Value::String(value)) => Ok(Some(value)),
        Some(_) => Err(ParseError::new(
            Format::WaveDrom,
            format!("`{key}` must be a string"),
            None,
            source,
        )),
    }
}

fn take_optional_number(
    object: &mut Map<String, Value>,
    key: &str,
    source: &str,
) -> ParseResult<Option<f64>> {
    match object.remove(key) {
        None => Ok(None),
        Some(Value::Number(value)) => value.as_f64().map(Some).ok_or_else(|| {
            ParseError::new(
                Format::WaveDrom,
                format!("`{key}` is outside the supported numeric range"),
                None,
                source,
            )
        }),
        Some(_) => Err(ParseError::new(
            Format::WaveDrom,
            format!("`{key}` must be numeric"),
            None,
            source,
        )),
    }
}

fn take_optional_integer(
    object: &mut Map<String, Value>,
    key: &str,
    source: &str,
) -> ParseResult<Option<i64>> {
    match object.remove(key) {
        None => Ok(None),
        Some(Value::Number(value)) => value.as_i64().map(Some).ok_or_else(|| {
            ParseError::new(
                Format::WaveDrom,
                format!("`{key}` must be an integer"),
                None,
                source,
            )
        }),
        Some(_) => Err(ParseError::new(
            Format::WaveDrom,
            format!("`{key}` must be an integer"),
            None,
            source,
        )),
    }
}
