use std::{path::PathBuf, str::FromStr};

use crate::{
    agi_version::AGIVersion,
    logic::{
        asm::expressions::ParsedLogicArgument,
        logic_script::{
            compile::{
                diagnostics::{LogicScriptDiagnostic, LogicScriptDiagnosticSeverity},
                errors::CompilationError,
            },
            directives::Directive,
            identifiers::IdentifierMap,
            locations::WithLocation,
            parsing::logic_script_parser,
            statements::{LogicScriptIfStatement, LogicScriptStatement, LogicScriptStatementBody},
        },
    },
    resources::file_provider::FileProvider,
};

fn preprocess_statement<FP: FileProvider>(
    statement: &LogicScriptStatement<WithLocation<ParsedLogicArgument>>,
    script_path: &str,
    identifier_map: &mut IdentifierMap,
    agi_version: &AGIVersion,
    file_provider: &FP,
) -> Result<
    Box<dyn Iterator<Item = LogicScriptStatement<WithLocation<ParsedLogicArgument>>>>,
    CompilationError,
> {
    let mut preprocess_substatements =
        |substatements: &[LogicScriptStatement<WithLocation<ParsedLogicArgument>>]| {
            Ok::<_, CompilationError>(
                substatements
                    .iter()
                    .map(|stmt| {
                        preprocess_statement(
                            stmt,
                            script_path,
                            identifier_map,
                            agi_version,
                            file_provider,
                        )
                    })
                    .collect::<Result<Vec<_>, CompilationError>>()?
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>(),
            )
        };

    match &statement.body {
        LogicScriptStatementBody::Directive(body) => match &body.directive {
            Directive::Define { identifier, value } => {
                identifier_map.define(identifier.name.clone(), value)?;
                return Ok(Box::new(std::iter::empty()));
            }
            Directive::Include { filename } => {
                let path = PathBuf::from_str(script_path)
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join(filename.value());
                let source_code = file_provider
                    .read_file_utf8(path.as_os_str().to_string_lossy().into_owned().as_str())?;
                let raw_program = parse_logic_script_raw(source_code.as_str(), agi_version)?;
                return Ok(Box::new(
                    preprocess_substatements(&raw_program)?.into_iter(),
                ));
            }
            _ => {}
        },
        LogicScriptStatementBody::IfStatement(body) => {
            return Ok(Box::new(std::iter::once(LogicScriptStatement {
                body: LogicScriptStatementBody::IfStatement(LogicScriptIfStatement {
                    conditions: body.conditions.clone(),
                    then_statements: preprocess_substatements(
                        body.then_statements
                            .iter()
                            .cloned()
                            .map(|stmt| *stmt)
                            .collect::<Vec<_>>()
                            .as_slice(),
                    )?
                    .into_iter()
                    .map(Box::new)
                    .collect(),
                    else_statements: preprocess_substatements(
                        body.else_statements
                            .iter()
                            .cloned()
                            .map(|stmt| *stmt)
                            .collect::<Vec<_>>()
                            .as_slice(),
                    )?
                    .into_iter()
                    .map(Box::new)
                    .collect(),
                    if_keyword: body.if_keyword.clone(),
                    else_keyword: body.else_keyword.clone(),
                }),
                label: statement.label.clone(),
            })));
        }
        _ => {}
    }

    Ok(Box::new(std::iter::once(statement.clone())))
}

pub fn preprocess_logic_script<FP: FileProvider>(
    raw_program: &[LogicScriptStatement<WithLocation<ParsedLogicArgument>>],
    script_path: &str,
    agi_version: &AGIVersion,
    file_provider: &FP,
) -> Result<
    (
        Vec<LogicScriptStatement<WithLocation<ParsedLogicArgument>>>,
        IdentifierMap,
    ),
    CompilationError,
> {
    let mut identifier_map = IdentifierMap::builtins();
    Ok((
        raw_program
            .iter()
            .map(|stmt| {
                preprocess_statement(
                    stmt,
                    script_path,
                    &mut identifier_map,
                    agi_version,
                    file_provider,
                )
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect(),
        identifier_map,
    ))
}

pub fn parse_logic_script_raw(
    source_code: &str,
    agi_version: &AGIVersion,
) -> Result<Vec<LogicScriptStatement<WithLocation<ParsedLogicArgument>>>, CompilationError> {
    let raw_program = logic_script_parser::program(source_code)?;
    let raw_program = raw_program
        .into_iter()
        .map(|stmt| *stmt)
        .collect::<Vec<_>>();
    let diagnostics = raw_program
        .iter()
        .flat_map(|stmt| LogicScriptDiagnostic::for_statement(stmt, agi_version))
        .collect::<Vec<_>>();

    if diagnostics
        .iter()
        .any(|d| d.severity == LogicScriptDiagnosticSeverity::Error)
    {
        return Err(CompilationError::FailedDiagnostics(diagnostics));
    }

    Ok(raw_program)
}
