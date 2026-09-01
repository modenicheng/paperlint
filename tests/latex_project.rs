use paperlint::{config::DefaultConfig, latex::parser};
use std::{fs, path::Path, path::PathBuf};
use tempfile::tempdir;

fn write(path: &Path, text: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, text).unwrap();
}

#[test]
fn tree_sitter_latex_language_links_and_parses() {
    let mut parser = tree_sitter::Parser::new();
    let language = tree_sitter_latex::LANGUAGE.into();
    parser.set_language(&language).unwrap();
    let tree = parser.parse(br"\input{child}", None).unwrap();
    assert!(!tree.root_node().has_error());
}

#[test]
fn parses_stdin_source_in_memory_with_virtual_spans() {
    let text = "前文。Github 后文。\n".to_string();
    let document = parser::parse_stdin(text.clone(), &DefaultConfig::load().latex).unwrap();

    assert_eq!(document.entry, PathBuf::from("<stdin>"));
    assert_eq!(document.root, PathBuf::new());
    assert_eq!(document.sources.len(), 1);
    assert_eq!(document.sources[0].path, PathBuf::from("<stdin>"));
    assert_eq!(document.sources[0].text, text);
    assert!(
        document
            .blocks
            .iter()
            .any(|block| block.text.contains("Github"))
    );
}

#[test]
fn stdin_include_is_a_contextual_parse_error() {
    let error = parser::parse_stdin(
        "\\input{chapter}\n".to_string(),
        &DefaultConfig::load().latex,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        parser::ParseError::StdinInclude { requested }
            if requested == *"chapter"
    ));
}

#[test]
fn recursively_expands_latex_includes_in_reading_order() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    write(&main, "开头。\\input{chapters/one}结尾。");
    write(
        &dir.path().join("chapters/one.tex"),
        "第一章。\\include{nested}",
    );
    write(&dir.path().join("chapters/nested.tex"), "嵌套正文。");

    let document = parser::parse(main.clone(), &DefaultConfig::load().latex).unwrap();
    let logical = document
        .blocks
        .iter()
        .map(|block| block.text.as_str())
        .collect::<String>();
    let positions =
        ["开头", "第一章", "嵌套正文", "结尾"].map(|marker| logical.find(marker).unwrap());
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(document.sources.len(), 3);
    for path in [
        main,
        dir.path().join("chapters/one.tex"),
        dir.path().join("chapters/nested.tex"),
    ] {
        let canonical = path.canonicalize().unwrap();
        assert_eq!(
            document
                .sources
                .iter()
                .filter(|source| source.path == canonical)
                .count(),
            1
        );
    }
}

#[test]
fn recognizes_all_latex_include_commands_and_extensionless_tex_paths() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    write(
        &main,
        "\\input{a}\\include{b}\\subfile{c}\\subfileinclude{d}",
    );
    for (name, marker) in [("a", "甲。"), ("b", "乙。"), ("c", "丙。"), ("d", "丁。")] {
        write(&dir.path().join(format!("{name}.tex")), marker);
    }
    let document = parser::parse(main, &DefaultConfig::load().latex).unwrap();
    let logical = document
        .blocks
        .iter()
        .map(|b| b.text.as_str())
        .collect::<String>();
    for marker in ["甲", "乙", "丙", "丁"] {
        assert_eq!(logical.matches(marker).count(), 1);
    }
}

#[test]
fn include_cycle_is_a_contextual_parse_error() {
    let dir = tempdir().unwrap();
    let a = dir.path().join("a.tex");
    write(&a, "\\input{b}");
    write(&dir.path().join("b.tex"), "\\input{a}");
    let error = parser::parse(a, &DefaultConfig::load().latex)
        .unwrap_err()
        .to_string();
    assert!(error.contains("cycle"));
    assert!(error.contains("a.tex") && error.contains("b.tex"));
}

#[test]
fn resolves_includes_relative_to_the_including_file() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    write(&main, "\\input{parts/one}");
    write(&dir.path().join("parts/one.tex"), "\\input{nested/two}");
    write(&dir.path().join("parts/nested/two.tex"), "相对路径正文。");
    let document = parser::parse(main, &DefaultConfig::load().latex).unwrap();
    assert!(
        document
            .blocks
            .iter()
            .any(|b| b.text.contains("相对路径正文"))
    );
}

#[test]
fn missing_include_has_including_file_and_request_context() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    write(&main, "\\input{missing/chapter}");
    let error = parser::parse(main, &DefaultConfig::load().latex)
        .unwrap_err()
        .to_string();
    assert!(error.contains("main.tex"));
    assert!(error.contains("missing/chapter"));
}

#[test]
fn duplicate_completed_file_is_loaded_once() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    write(&main, "\\input{child}中间。\\input{child}");
    write(&dir.path().join("child.tex"), "唯一正文。");
    let document = parser::parse(main, &DefaultConfig::load().latex).unwrap();
    let logical = document
        .blocks
        .iter()
        .map(|b| b.text.as_str())
        .collect::<String>();
    assert_eq!(logical.matches("唯一正文").count(), 1);
    assert_eq!(document.sources.len(), 2);
}

#[test]
fn extraction_retains_prose_and_excludes_non_prose_ast_nodes() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    write(
        &main,
        r#"\documentclass{classsecret}
\usepackage{packagesecret}
\section{标题文字}
正文开始。我们采用\textbf{BERT模型}。
\begin{figure}图示正文。\caption{说明文字}\includegraphics{graphicsecret.png}\end{figure}
% commentsecret
\begin{comment}blockcommentsecret\end{comment}
$mathsecret$ \(inlinemathsecret\) \[displaymathsecret\]
\begin{equation}equationsecret\end{equation}
\begin{lstlisting}listingsecret\end{lstlisting}
\cite{citesecret} \label{labelsecret} \ref{refsecret}
\newcommand{\foo}{definitionsecret}
\bibliography{bibsecret}
"#,
    );
    let document = parser::parse(main, &DefaultConfig::load().latex).unwrap();
    let logical = document
        .blocks
        .iter()
        .map(|b| b.text.as_str())
        .collect::<String>();
    for retained in [
        "标题文字",
        "正文开始。",
        "我们采用BERT模型。",
        "图示正文。",
        "说明文字",
    ] {
        assert!(
            logical.contains(retained),
            "missing {retained:?} in {logical:?}"
        );
    }
    for excluded in [
        "classsecret",
        "packagesecret",
        "graphicsecret",
        "commentsecret",
        "blockcommentsecret",
        "mathsecret",
        "inlinemathsecret",
        "displaymathsecret",
        "equationsecret",
        "listingsecret",
        "citesecret",
        "labelsecret",
        "refsecret",
        "definitionsecret",
        "bibsecret",
    ] {
        assert!(
            !logical.contains(excluded),
            "found {excluded:?} in {logical:?}"
        );
    }
}

#[test]
fn excludes_bibliography_commands_and_thebibliography_contents() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    write(
        &main,
        r#"Visible prose before the references.
\addbibresource{BIBRESOURCENASA.bib}
\bibliographystyle{BIBSTYLENASA}
\bibliography{BIBTEXNASA}
\printbibliography[heading=BIBHEADINGNASA]
\begin{thebibliography}{BIBWIDTHNASA}
\bibitem{BIBKEYNASA} Author NASA. \emph{Title API}.
\end{thebibliography}
Visible prose after the references.
"#,
    );

    let document = parser::parse(main, &DefaultConfig::load().latex).unwrap();
    let logical = document
        .blocks
        .iter()
        .map(|block| block.text.as_str())
        .collect::<String>();

    for retained in [
        "Visible prose before the references.",
        "Visible prose after the references.",
    ] {
        assert!(
            logical.contains(retained),
            "missing {retained:?} in {logical:?}"
        );
    }
    for bibliography_text in [
        "BIBRESOURCENASA",
        "BIBSTYLENASA",
        "BIBTEXNASA",
        "BIBHEADINGNASA",
        "BIBWIDTHNASA",
        "BIBKEYNASA",
        "Author NASA",
        "Title API",
    ] {
        assert!(
            !logical.contains(bibliography_text),
            "found {bibliography_text:?} in {logical:?}"
        );
    }
    assert_eq!(document.sources.len(), 1, "BibTeX source was loaded");
}

#[test]
fn excludes_standalone_bibitem_entries_until_the_next_document_section() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    write(
        &main,
        r#"Visible prose before the references.
\bibitem{BIBKEYNASA} Author NASA. API title.
\bibitem[Optional XML label]{BIBKEYXML} Another XML entry.
\section{Visible section after references}
Visible prose after the references.
"#,
    );

    let document = parser::parse(main, &DefaultConfig::load().latex).unwrap();
    let logical = document
        .blocks
        .iter()
        .map(|block| block.text.as_str())
        .collect::<String>();

    for retained in [
        "Visible prose before the references.",
        "Visible section after references",
        "Visible prose after the references.",
    ] {
        assert!(
            logical.contains(retained),
            "missing {retained:?} in {logical:?}"
        );
    }
    for bibliography_text in [
        "BIBKEYNASA",
        "Author NASA",
        "API title",
        "Optional XML label",
        "BIBKEYXML",
        "Another XML entry",
    ] {
        assert!(
            !logical.contains(bibliography_text),
            "found {bibliography_text:?} in {logical:?}"
        );
    }
}

#[test]
fn excludes_specialized_definition_fields_and_color_names() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    write(
        &main,
        r#"\color{NASA} ordinary prose.
\textcolor{secretblue}{visible text}.
\definecolor{COLORSECRET}{rgb}{1,0,0}
\let\alias=\NASA
\DeclarePairedDelimiter{\pair}{LEFTSECRET}{RIGHTSECRET}
\newtheorem{THEOREMSECRET}{TITLESECRET}
\def\legacy{OLDSECRET}
retained ending.
"#,
    );

    let document = parser::parse(main, &DefaultConfig::load().latex).unwrap();
    let logical = document
        .blocks
        .iter()
        .map(|block| block.text.as_str())
        .collect::<String>();

    for retained in ["ordinary prose", "visible text", "retained ending"] {
        assert!(
            logical.contains(retained),
            "missing {retained:?} in {logical:?}"
        );
    }
    for excluded in [
        "NASA",
        "secretblue",
        "COLORSECRET",
        "alias",
        "LEFTSECRET",
        "RIGHTSECRET",
        "THEOREMSECRET",
        "TITLESECRET",
        "legacy",
        "OLDSECRET",
    ] {
        assert!(
            !logical.contains(excluded),
            "found {excluded:?} in {logical:?}"
        );
    }
}

#[test]
fn excludes_all_definition_reference_and_counter_metadata_subtrees() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    write(
        &main,
        r#"\newacronym[AcronymOptionMeta]{AcronymNameMeta}{AcronymShortMeta}{AcronymLongMeta}
\acrshort{AcronymReferenceMeta}
\newglossaryentry{GlossaryNameMeta}{name={GlossaryNameFieldMeta},description={GlossaryDescriptionMeta}}
\gls{GlossaryReferenceMeta}
\definecolorset{ColorModelMeta}{ColorHeadMeta}{ColorTailMeta}{ColorSpecMeta}
\newcounter{CounterDeclarationMeta}[CounterSuperMeta]
\counterwithin{CounterWithinMeta}{CounterSuperWithinMeta}
\counterwithout{CounterWithoutMeta}{CounterSuperWithoutMeta}
\setcounter{CounterDefinitionMeta}{7}
\addtocounter{CounterAdditionMeta}{1}
\stepcounter{CounterIncrementMeta}
\refstepcounter{CounterRefIncrementMeta}
\arabic{CounterTypesettingMeta}
Ordinary prose remains visible.
"#,
    );

    let document = parser::parse(main, &DefaultConfig::load().latex).unwrap();
    let logical = document
        .blocks
        .iter()
        .map(|block| block.text.as_str())
        .collect::<String>();

    assert!(logical.contains("Ordinary prose remains visible."));
    for metadata in [
        "AcronymOptionMeta",
        "AcronymNameMeta",
        "AcronymShortMeta",
        "AcronymLongMeta",
        "AcronymReferenceMeta",
        "GlossaryNameMeta",
        "GlossaryNameFieldMeta",
        "GlossaryDescriptionMeta",
        "GlossaryReferenceMeta",
        "ColorModelMeta",
        "ColorHeadMeta",
        "ColorTailMeta",
        "ColorSpecMeta",
        "CounterDeclarationMeta",
        "CounterSuperMeta",
        "CounterWithinMeta",
        "CounterSuperWithinMeta",
        "CounterWithoutMeta",
        "CounterSuperWithoutMeta",
        "CounterDefinitionMeta",
        "CounterAdditionMeta",
        "CounterIncrementMeta",
        "CounterRefIncrementMeta",
        "CounterTypesettingMeta",
    ] {
        assert!(
            !logical.contains(metadata),
            "found {metadata:?} in {logical:?}"
        );
    }
}

#[test]
fn excludes_list_labels_and_resource_reference_metadata() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    write(
        &main,
        r#"\begin{itemize}
\item[LISTLABELNASA] Ordinary list prose remains visible.
\end{itemize}
\includesvg[title={SVGNASA}]{svgsecret}
\includeinkscape[title={INKSCAPENASA}]{inkscapesecret}
\newlabel{labelsecret}{LABELNUMBERNASA}
\crefrange{fromsecret}{tosecret}
\bibliographystyle{BIBSTYLENASA}
\bibliography{BIBTEXNASA}
\verbatiminput{VERBATIMNASA}
\import{IMPORTDIRNASA}{IMPORTFILENASA}
\usetikzlibrary{TIKZLIBNASA}
Visible ending remains.
"#,
    );

    let document = parser::parse(main, &DefaultConfig::load().latex).unwrap();
    let logical = document
        .blocks
        .iter()
        .map(|block| block.text.as_str())
        .collect::<String>();

    for retained in [
        "Ordinary list prose remains visible.",
        "Visible ending remains.",
    ] {
        assert!(
            logical.contains(retained),
            "missing {retained:?} in {logical:?}"
        );
    }
    for metadata in [
        "LISTLABELNASA",
        "SVGNASA",
        "INKSCAPENASA",
        "LABELNUMBERNASA",
        "fromsecret",
        "tosecret",
        "BIBSTYLENASA",
        "BIBTEXNASA",
        "VERBATIMNASA",
        "IMPORTDIRNASA",
        "IMPORTFILENASA",
        "TIKZLIBNASA",
    ] {
        assert!(
            !logical.contains(metadata),
            "found {metadata:?} in {logical:?}"
        );
    }
    assert_eq!(document.sources.len(), 1, "import metadata was loaded");
}

#[test]
fn paragraph_separators_and_titles_form_distinct_blocks() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    write(
        &main,
        "\\section{A short title}\nfirst short paragraph\n\nsecond short paragraph.\n",
    );

    let document = parser::parse(main, &DefaultConfig::load().latex).unwrap();
    let nonempty: Vec<_> = document
        .blocks
        .iter()
        .map(|block| block.text.trim())
        .filter(|text| !text.is_empty())
        .collect();

    assert_eq!(
        nonempty,
        [
            "A short title",
            "first short paragraph",
            "second short paragraph."
        ]
    );
}

#[test]
fn a_single_crlf_wrap_stays_in_one_paragraph_block() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    write(&main, "one wrapped\r\nline.");

    let document = parser::parse(main, &DefaultConfig::load().latex).unwrap();
    let nonempty: Vec<_> = document
        .blocks
        .iter()
        .map(|block| block.text.trim())
        .filter(|text| !text.is_empty())
        .collect();

    assert_eq!(nonempty, ["one wrapped\r\nline."]);
}

#[test]
fn maps_utf8_child_match_to_original_latex_bytes() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    let child = dir.path().join("child.tex");
    write(&main, "开头。\\input{child}结尾。");
    write(&child, "前缀\\textbf{中文问题}后缀。");
    let document = parser::parse(main, &DefaultConfig::load().latex).unwrap();
    let block = document
        .blocks
        .iter()
        .find(|b| b.text.contains("中文问题"))
        .unwrap();
    let start = block.text.find("中文问题").unwrap();
    let span = document
        .source_span(block, start..start + "中文问题".len())
        .unwrap();
    let canonical = child.canonicalize().unwrap();
    let source = document.source(&canonical).unwrap();
    assert_eq!(span.file, canonical);
    assert_eq!(&source.text[span.start..span.end], "中文问题");
    assert_eq!((span.line, span.column), (1, 11));
}

#[test]
fn absolute_fallback_paths_have_one_posix_root_separator() {
    assert_eq!(
        paperlint::output::path::display_path(Path::new("/outside/file.tex"), Path::new("/paper"),),
        "/outside/file.tex"
    );
}

#[test]
fn bare_cr_mapping_derives_line_and_column_from_original_source() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    write(&main, "第一行。\r第二行\\textbf{目标词}。\r");

    let document = parser::parse(main.clone(), &DefaultConfig::load().latex).unwrap();
    let block = document
        .blocks
        .iter()
        .find(|block| block.text.contains("目标词"))
        .unwrap();
    let start = block.text.find("目标词").unwrap();
    let span = document
        .source_span(block, start..start + "目标词".len())
        .unwrap();
    let source = document.source(&main.canonicalize().unwrap()).unwrap();

    assert_eq!(&source.text[span.start..span.end], "目标词");
    assert_eq!((span.line, span.column), (2, 12));
    assert!(source.text.contains('\r'));
    assert!(!source.text.contains('\n'));
}

#[test]
fn crlf_mapping_preserves_original_byte_offsets() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.tex");
    write(&main, "第一行。\r\n第二行\\textbf{目标词}。\r\n");
    let document = parser::parse(main.clone(), &DefaultConfig::load().latex).unwrap();
    let block = document
        .blocks
        .iter()
        .find(|b| b.text.contains("目标词"))
        .unwrap();
    let start = block.text.find("目标词").unwrap();
    let span = document
        .source_span(block, start..start + "目标词".len())
        .unwrap();
    let source = document.source(&main.canonicalize().unwrap()).unwrap();
    assert_eq!(&source.text[span.start..span.end], "目标词");
    assert_eq!((span.line, span.column), (2, 12));
    assert!(source.text.contains("\r\n"));
}
