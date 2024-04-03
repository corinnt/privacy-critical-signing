#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_middle; 
extern crate rustc_span;
extern crate rustc_data_structures; 
extern crate rustc_query_system; 

use clippy_utils::sym;
use clippy_utils::diagnostics::span_lint_and_help;

use rustc_lint::LateLintPass;
use rustc_ast::ast::LitKind;
use rustc_span::symbol::Symbol;

use rustc_hir::Expr;
use rustc_hir::ExprKind;
use rustc_hir::def::Res; 

use rustc_middle::ty::{TyCtxt, InstanceDef};

use rustc_data_structures::stable_hasher::{HashStable, StableHasher};
use rustc_query_system::ich::StableHashingContext; 

use if_chain::if_chain; 
use base64::{engine::general_purpose, Engine as _};

use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};


dylint_linting::declare_late_lint! {
    /// ### What it does
    ///
    /// ### Why is this bad?
    ///
    /// ### Known problems
    /// Remove if none.
    ///
    /// ### Example
    /// ```rust
    /// // example code where a warning is issued
    /// ```
    /// Use instead:
    /// ```rust
    /// // example code that does not raise a warning
    /// ```
    pub SIGNATURE_LINT,
    Warn,
    "description goes here"
}

impl<'tcx> LateLintPass<'tcx> for SignatureLint {
    fn check_expr(&mut self, cx: &rustc_lint::LateContext<'tcx>, expr: &'_ rustc_hir::Expr<'_>) { 
        let fn_path: Vec<Symbol> = vec![sym!(alohomora), 
                                        sym!(pcr), 
                                        sym!(PrivacyCriticalRegion), 
                                        sym!(new)];
        
        if let ExprKind::Call(maybe_path, args) = &expr.kind {
            if is_fn_call(cx, maybe_path, fn_path){
                assert!(args.len() == 3); // 3 args to constructor of PrivacyCriticalRegion
                if let ExprKind::Closure(closure) = args[0].kind {
                    let closure_body = cx.tcx.hir().body(closure.body);
                    let pcr_src = cx.tcx
                                .sess
                                .source_map()
                                .span_to_snippet(closure_body.value.span)
                                .unwrap(); 
                    let pcr_hash = get_mir_hash(cx.tcx, closure);
                    println!("mir hash: {}", pcr_hash); 
                    //These args to PrivacyCriticalRegion::new will be of type Signature
                    let author = extract_from_signature_struct(args[1].kind);
                    let reviewer = extract_from_signature_struct(args[2].kind);

                    let author_identity_checked = check_identity(&pcr_hash, &author);
                    let reviewer_identity_checked = check_identity(&pcr_hash, &reviewer);
    
                    if author_identity_checked.is_err() || reviewer_identity_checked.is_err() {
                        let mut help_msg = String::new();
                        push_id_error(&mut help_msg, "author", author_identity_checked); 
                        push_id_error(&mut help_msg, "reviewer", reviewer_identity_checked); 
            
                        let file_loc = cx.tcx
                                    .sess
                                    .source_map()
                                    .span_to_diagnostic_string(closure_body.value.span)
                                    .replace("/", "_")
                                    .replace(" ", "-"); 

                        // add timestamp to avoid overwriting between runs or matching filepaths after substitutions
                        let timestamp = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
                                    .as_millis(); 

                        let pcr_file_name = format!("./pcr/{}_{}.rs", file_loc, timestamp);
                        let hash_file_name = format!("./pcr/{}_hash_{}.rs", file_loc, timestamp); 
                    
                        help_msg.push_str(
                            format!(
                                "written the hash of privacy-critical region into the file for signing: {}",
                                hash_file_name
                            )
                            .as_str(),
                        ); 
                        
                        if !Path::exists("./pcr/".as_ref()) {
                            fs::create_dir("./pcr/").unwrap();
                        }
                    
                        fs::write(pcr_file_name, pcr_src).unwrap();
                        fs::write(hash_file_name, pcr_hash).unwrap();
                    
                        span_lint_and_help(
                            cx,
                            SIGNATURE_LINT,
                            expr.span,
                            "badly-signed privacy-critical region, might be a source of privacy-related bugs",
                            None,
                            help_msg.as_str()
                        );
                    } 
                } else {
                    panic!("Attempting to hash something different from a Closure.")
                }
            }
        }
    }
}
    
// Returns true if the given Expression is of ExprKind::Path & path resolves to given fn_pat
fn is_fn_call(cx: &rustc_lint::LateContext, maybe_path: &Expr, fn_path: Vec<Symbol>) -> bool {
    if_chain! {
        if let ExprKind::Path(ref qpath) = maybe_path.kind; 
        if let Res::Def(_kind, def_id) = cx.typeck_results().qpath_res(qpath, maybe_path.hir_id); 
        if cx.match_def_path(def_id, &fn_path); 
        then {
            true
        } else {
            false
        }
    }
}

// Given an ExprKind that may be a Signature struct, returns fields (username, signature) 
fn extract_from_signature_struct(maybe_struct: ExprKind) -> (String, String) {
    if let ExprKind::Struct(_, fields, _) = maybe_struct {
        assert!(fields.len() == 2);

        let username = if let ExprKind::Lit(spanned) = fields[0].expr.kind {
            if let LitKind::Str(username, _) = spanned.node {
                String::from(username.as_str())
            } else {
                panic!("Attempting to use a non-string author username.");
            }
        } else {
            panic!("Attempting to use a non-string author username.");
        };

        let signature = if let ExprKind::Lit(spanned) = fields[1].expr.kind {
            if let LitKind::Str(signature, _) = spanned.node {
                String::from(signature.as_str())
            } else {
                panic!("Attempting to use a non-string author username.");
            }
        } else {
            panic!("Attempting to use a non-string author username.");
        };

        (username, signature)
    } else {
        panic!("Invalid use of privacy-critical region.");
    }
}

// Given a Closure, returns the (String) StableHash of its MIR Body 
fn get_mir_hash<'a>(tcx: TyCtxt, closure: &rustc_hir::Closure) -> String {
    let def_id: rustc_hir::def_id::DefId = closure.def_id.to_def_id(); 

    //try using optimized MIR once Span hashing is fixed - optim is dropping the unused fields ?
    //let mir_body = tcx.optimized_mir(def_id); 

    let instance_def = InstanceDef::Item(def_id); 
    let mir_body: &rustc_middle::mir::Body = tcx.instance_mir(instance_def); 
    let mut new_mir_body = mir_body.clone(); 

    // currently replaces span field on root MIR Body, 
    // but they're also nested in the terminators of the basic_blocks
    let mod_span = tcx.hir().root_module().spans.inner_span; 
    new_mir_body.span = mod_span; 
  
    // StableHasher is always instantiated with the same State -> deterministic hash
    let mut hcx = StableHashingContext::new(tcx.sess, tcx.untracked()); 
    let mut hasher = StableHasher::new(); 
    new_mir_body.hash_stable(&mut hcx, &mut hasher); 

    let hash_tuple: (u64, u64) = hasher.finalize(); 
    let mir_hash = format!("{:x} {:x}", hash_tuple.0, hash_tuple.1); 
    mir_hash
}

fn check_identity(pcr: &String, identity: &(String, String)) -> Result<(), String> {
    let (username, signature) = identity;
    let keys = get_github_keys(username)
        .lines()
        .map(|line| format!("{}@github.com {}", username, line))
        .collect::<Vec<_>>()
        .join("\n");

    let decoded_signature_bytes = general_purpose::STANDARD_NO_PAD
        .decode(signature)
        .map_err(|err| err.to_string())?;
    let decoded_signature =
        std::str::from_utf8(decoded_signature_bytes.as_slice()).map_err(|err| err.to_string())?;

    fs::write("/tmp/allowed_signers", keys).map_err(|err| err.to_string())?;
    fs::write("/tmp/signature", decoded_signature).map_err(|err| err.to_string())?;
    fs::write("/tmp/plaintext", pcr).map_err(|err| err.to_string())?;

    let command_str = format!("/usr/bin/ssh-keygen -Y verify -f /tmp/allowed_signers -I {}@github.com -n file -s /tmp/signature < /tmp/plaintext", username);

    let mut command = Command::new("zsh");
    command.args(["-c", command_str.as_str()]);
    let output = command.output();

    fs::remove_file("/tmp/allowed_signers").map_err(|err| err.to_string())?;
    fs::remove_file("/tmp/signature").map_err(|err| err.to_string())?;
    fs::remove_file("/tmp/plaintext").map_err(|err| err.to_string())?;

    match output {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                Err(String::from(
                    std::str::from_utf8(output.stderr.as_slice()).map_err(|err| err.to_string())?,
                ))
            }
        }
        Err(err) => Err(err.to_string()),
    }
}

fn get_github_keys(username: &String) -> String {
    reqwest::blocking::get(format!("https://github.com/{}.keys", username))
        .unwrap()
        .text()
        .unwrap()
}

fn push_id_error(msg: &mut String, id: &str, res: Result<(), String>) {
    if res.is_err() {
        msg.push_str(
            format!(
                "could not verify {}'s signature: {}\n",
                id,
                res.unwrap_err().trim()
            )
            .as_str(),
        );
    }
}
 

#[test]
fn ui() {
    dylint_testing::ui_test(
        env!("CARGO_PKG_NAME"),
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui"),
    );
}
