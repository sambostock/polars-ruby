mod conversion;
mod dataframe;
mod error;
mod file;
mod lazy;
mod series;

use conversion::get_df;
use dataframe::RbDataFrame;
use error::{RbPolarsErr, RbValueError};
use lazy::dataframe::{RbLazyFrame, RbLazyGroupBy};
use lazy::dsl::{RbExpr, RbWhen, RbWhenThen};
use magnus::{
    define_module, function, memoize, method, prelude::*, Error, RArray, RClass, RModule,
};
use polars::error::PolarsResult;
use polars::frame::DataFrame;
use polars::functions::{diag_concat_df, hor_concat_df};
use series::RbSeries;

type RbResult<T> = Result<T, Error>;

fn module() -> RModule {
    *memoize!(RModule: define_module("Polars").unwrap())
}

fn series() -> RClass {
    *memoize!(RClass: module().define_class("Series", Default::default()).unwrap())
}

#[magnus::init]
fn init() -> RbResult<()> {
    let module = module();
    module.define_singleton_method("_concat_df", function!(concat_df, 1))?;
    module.define_singleton_method("_diag_concat_df", function!(rb_diag_concat_df, 1))?;
    module.define_singleton_method("_hor_concat_df", function!(rb_hor_concat_df, 1))?;

    let class = module.define_class("RbDataFrame", Default::default())?;
    class.define_singleton_method("new", function!(RbDataFrame::init, 1))?;
    class.define_singleton_method("read_csv", function!(RbDataFrame::read_csv, 2))?;
    class.define_singleton_method("read_parquet", function!(RbDataFrame::read_parquet, 1))?;
    class.define_singleton_method("read_hash", function!(RbDataFrame::read_hash, 1))?;
    class.define_singleton_method("read_json", function!(RbDataFrame::read_json, 1))?;
    class.define_singleton_method("read_ndjson", function!(RbDataFrame::read_ndjson, 1))?;
    class.define_method("write_json", method!(RbDataFrame::write_json, 3))?;
    class.define_method("write_ndjson", method!(RbDataFrame::write_ndjson, 1))?;
    class.define_method("write_csv", method!(RbDataFrame::write_csv, 10))?;
    class.define_method("write_parquet", method!(RbDataFrame::write_parquet, 5))?;
    class.define_method("rechunk", method!(RbDataFrame::rechunk, 0))?;
    class.define_method("to_s", method!(RbDataFrame::to_s, 0))?;
    class.define_method("columns", method!(RbDataFrame::columns, 0))?;
    class.define_method("dtypes", method!(RbDataFrame::dtypes, 0))?;
    class.define_method("shape", method!(RbDataFrame::shape, 0))?;
    class.define_method("height", method!(RbDataFrame::height, 0))?;
    class.define_method("width", method!(RbDataFrame::width, 0))?;
    class.define_method("select_at_idx", method!(RbDataFrame::select_at_idx, 1))?;
    class.define_method("column", method!(RbDataFrame::column, 1))?;
    class.define_method("sort", method!(RbDataFrame::sort, 3))?;
    class.define_method("head", method!(RbDataFrame::head, 1))?;
    class.define_method("tail", method!(RbDataFrame::tail, 1))?;
    class.define_method("frame_equal", method!(RbDataFrame::frame_equal, 2))?;
    class.define_method("_clone", method!(RbDataFrame::clone, 0))?;
    class.define_method("lazy", method!(RbDataFrame::lazy, 0))?;
    class.define_method("mean", method!(RbDataFrame::mean, 0))?;
    class.define_method("null_count", method!(RbDataFrame::null_count, 0))?;

    let class = module.define_class("RbExpr", Default::default())?;
    class.define_method("*", method!(RbExpr::mul, 1))?;
    class.define_method("to_str", method!(RbExpr::to_str, 0))?;
    class.define_method("eq", method!(RbExpr::eq, 1))?;
    class.define_method("neq", method!(RbExpr::neq, 1))?;
    class.define_method("gt", method!(RbExpr::gt, 1))?;
    class.define_method("gt_eq", method!(RbExpr::gt_eq, 1))?;
    class.define_method("lt_eq", method!(RbExpr::lt_eq, 1))?;
    class.define_method("lt", method!(RbExpr::lt, 1))?;
    class.define_method("_alias", method!(RbExpr::alias, 1))?;
    class.define_method("is_not", method!(RbExpr::is_not, 0))?;
    class.define_method("is_null", method!(RbExpr::is_null, 0))?;
    class.define_method("is_not_null", method!(RbExpr::is_not_null, 0))?;
    class.define_method("min", method!(RbExpr::min, 0))?;
    class.define_method("max", method!(RbExpr::max, 0))?;
    class.define_method("mean", method!(RbExpr::mean, 0))?;
    class.define_method("median", method!(RbExpr::median, 0))?;
    class.define_method("sum", method!(RbExpr::sum, 0))?;
    class.define_method("n_unique", method!(RbExpr::n_unique, 0))?;
    class.define_method("unique", method!(RbExpr::unique, 0))?;
    class.define_method("unique_stable", method!(RbExpr::unique_stable, 0))?;
    class.define_method("first", method!(RbExpr::first, 0))?;
    class.define_method("last", method!(RbExpr::last, 0))?;
    class.define_method("list", method!(RbExpr::list, 0))?;
    class.define_method("count", method!(RbExpr::count, 0))?;
    class.define_method("sort_with", method!(RbExpr::sort_with, 2))?;
    class.define_method("sort_by", method!(RbExpr::sort_by, 2))?;
    class.define_method("fill_null", method!(RbExpr::fill_null, 1))?;
    class.define_method(
        "fill_null_with_strategy",
        method!(RbExpr::fill_null_with_strategy, 2),
    )?;
    class.define_method("fill_nan", method!(RbExpr::fill_nan, 1))?;
    class.define_method("drop_nulls", method!(RbExpr::drop_nulls, 0))?;
    class.define_method("drop_nans", method!(RbExpr::drop_nans, 0))?;
    class.define_method("filter", method!(RbExpr::filter, 1))?;
    class.define_method("reverse", method!(RbExpr::reverse, 0))?;
    class.define_method("std", method!(RbExpr::std, 1))?;
    class.define_method("var", method!(RbExpr::var, 1))?;
    class.define_method("tail", method!(RbExpr::tail, 1))?;
    class.define_method("head", method!(RbExpr::head, 1))?;
    class.define_method("over", method!(RbExpr::over, 1))?;
    class.define_method("_and", method!(RbExpr::_and, 1))?;
    class.define_method("_xor", method!(RbExpr::_xor, 1))?;
    class.define_method("_or", method!(RbExpr::_or, 1))?;
    class.define_method("product", method!(RbExpr::product, 0))?;
    class.define_method("str_lengths", method!(RbExpr::str_lengths, 0))?;
    class.define_method("str_contains", method!(RbExpr::str_contains, 2))?;
    class.define_method("prefix", method!(RbExpr::prefix, 1))?;
    class.define_method("suffix", method!(RbExpr::suffix, 1))?;
    class.define_method("interpolate", method!(RbExpr::interpolate, 0))?;

    // maybe add to different class
    class.define_singleton_method("col", function!(crate::lazy::dsl::col, 1))?;
    class.define_singleton_method("lit", function!(crate::lazy::dsl::lit, 1))?;
    class.define_singleton_method("arange", function!(crate::lazy::dsl::arange, 3))?;
    class.define_singleton_method("when", function!(crate::lazy::dsl::when, 1))?;

    let class = module.define_class("RbLazyFrame", Default::default())?;
    class.define_method(
        "optimization_toggle",
        method!(RbLazyFrame::optimization_toggle, 7),
    )?;
    class.define_method("collect", method!(RbLazyFrame::collect, 0))?;
    class.define_method("filter", method!(RbLazyFrame::filter, 1))?;
    class.define_method("select", method!(RbLazyFrame::select, 1))?;
    class.define_method("groupby", method!(RbLazyFrame::groupby, 2))?;
    class.define_method("join", method!(RbLazyFrame::join, 7))?;
    class.define_method("with_columns", method!(RbLazyFrame::with_columns, 1))?;

    let class = module.define_class("RbLazyGroupBy", Default::default())?;
    class.define_method("agg", method!(RbLazyGroupBy::agg, 1))?;

    let class = module.define_class("RbSeries", Default::default())?;
    class.define_singleton_method("new_opt_bool", function!(RbSeries::new_opt_bool, 3))?;
    class.define_singleton_method("new_opt_u8", function!(RbSeries::new_opt_u8, 3))?;
    class.define_singleton_method("new_opt_u16", function!(RbSeries::new_opt_u16, 3))?;
    class.define_singleton_method("new_opt_u32", function!(RbSeries::new_opt_u32, 3))?;
    class.define_singleton_method("new_opt_u64", function!(RbSeries::new_opt_u64, 3))?;
    class.define_singleton_method("new_opt_i8", function!(RbSeries::new_opt_i8, 3))?;
    class.define_singleton_method("new_opt_i16", function!(RbSeries::new_opt_i16, 3))?;
    class.define_singleton_method("new_opt_i32", function!(RbSeries::new_opt_i32, 3))?;
    class.define_singleton_method("new_opt_i64", function!(RbSeries::new_opt_i64, 3))?;
    class.define_singleton_method("new_opt_f32", function!(RbSeries::new_opt_f32, 3))?;
    class.define_singleton_method("new_opt_f64", function!(RbSeries::new_opt_f64, 3))?;
    class.define_singleton_method("new_str", function!(RbSeries::new_str, 3))?;
    class.define_method("is_sorted_flag", method!(RbSeries::is_sorted_flag, 0))?;
    class.define_method("is_sorted_reverse_flag", method!(RbSeries::is_sorted_reverse_flag, 0))?;
    class.define_method("rechunk", method!(RbSeries::rechunk, 1))?;
    class.define_method("bitand", method!(RbSeries::bitand, 1))?;
    class.define_method("bitor", method!(RbSeries::bitor, 1))?;
    class.define_method("bitxor", method!(RbSeries::bitxor, 1))?;
    class.define_method("chunk_lengths", method!(RbSeries::chunk_lengths, 0))?;
    class.define_method("name", method!(RbSeries::name, 0))?;
    class.define_method("rename", method!(RbSeries::rename, 1))?;
    class.define_method("dtype", method!(RbSeries::dtype, 0))?;
    class.define_method("inner_dtype", method!(RbSeries::inner_dtype, 0))?;
    class.define_method("set_sorted", method!(RbSeries::set_sorted, 1))?;
    class.define_method("mean", method!(RbSeries::mean, 0))?;
    class.define_method("max", method!(RbSeries::max, 0))?;
    class.define_method("min", method!(RbSeries::min, 0))?;
    class.define_method("sum", method!(RbSeries::sum, 0))?;
    class.define_method("n_chunks", method!(RbSeries::n_chunks, 0))?;
    class.define_method("append", method!(RbSeries::append, 1))?;
    class.define_method("extend", method!(RbSeries::extend, 1))?;
    class.define_method("new_from_index", method!(RbSeries::new_from_index, 2))?;
    class.define_method("filter", method!(RbSeries::filter, 1))?;
    class.define_method("add", method!(RbSeries::add, 1))?;
    class.define_method("sub", method!(RbSeries::sub, 1))?;
    class.define_method("mul", method!(RbSeries::mul, 1))?;
    class.define_method("div", method!(RbSeries::div, 1))?;
    class.define_method("rem", method!(RbSeries::rem, 1))?;
    class.define_method("sort", method!(RbSeries::sort, 1))?;
    class.define_method("value_counts", method!(RbSeries::value_counts, 1))?;
    class.define_method("arg_min", method!(RbSeries::arg_min, 0))?;
    class.define_method("arg_max", method!(RbSeries::arg_max, 0))?;
    class.define_method("take_with_series", method!(RbSeries::take_with_series, 1))?;
    class.define_method("null_count", method!(RbSeries::null_count, 0))?;
    class.define_method("has_validity", method!(RbSeries::has_validity, 0))?;
    class.define_method("sample_n", method!(RbSeries::sample_n, 4))?;
    class.define_method("sample_frac", method!(RbSeries::sample_frac, 4))?;
    class.define_method("series_equal", method!(RbSeries::series_equal, 3))?;
    class.define_method("eq", method!(RbSeries::eq, 1))?;
    class.define_method("neq", method!(RbSeries::neq, 1))?;
    class.define_method("gt", method!(RbSeries::gt, 1))?;
    class.define_method("gt_eq", method!(RbSeries::gt_eq, 1))?;
    class.define_method("lt", method!(RbSeries::lt, 1))?;
    class.define_method("lt_eq", method!(RbSeries::lt_eq, 1))?;
    class.define_method("not", method!(RbSeries::not, 0))?;
    class.define_method("to_s", method!(RbSeries::to_s, 0))?;
    class.define_method("len", method!(RbSeries::len, 0))?;
    class.define_method("to_a", method!(RbSeries::to_a, 0))?;
    class.define_method("median", method!(RbSeries::median, 0))?;
    class.define_method("_clone", method!(RbSeries::clone, 0))?;
    // rest
    class.define_method("cumsum", method!(RbSeries::cumsum, 1))?;
    class.define_method("cummax", method!(RbSeries::cummax, 1))?;
    class.define_method("cummin", method!(RbSeries::cummin, 1))?;
    class.define_method("slice", method!(RbSeries::slice, 2))?;

    let class = module.define_class("RbWhen", Default::default())?;
    class.define_method("_then", method!(RbWhen::then, 1))?;

    let class = module.define_class("RbWhenThen", Default::default())?;
    class.define_method("otherwise", method!(RbWhenThen::overwise, 1))?;

    Ok(())
}

fn concat_df(seq: RArray) -> RbResult<RbDataFrame> {
    let mut iter = seq.each();
    let first = iter.next().unwrap()?;

    let first_rdf = get_df(first)?;
    let identity_df = first_rdf.slice(0, 0);

    let mut rdfs: Vec<PolarsResult<DataFrame>> = vec![Ok(first_rdf)];

    for item in iter {
        let rdf = get_df(item?)?;
        rdfs.push(Ok(rdf));
    }

    let identity = Ok(identity_df);

    let df = rdfs
        .into_iter()
        .fold(identity, |acc: PolarsResult<DataFrame>, df| {
            let mut acc = acc?;
            acc.vstack_mut(&df?)?;
            Ok(acc)
        })
        .map_err(RbPolarsErr::from)?;

    Ok(df.into())
}

fn rb_diag_concat_df(seq: RArray) -> RbResult<RbDataFrame> {
    let mut dfs = Vec::new();
    for item in seq.each() {
        dfs.push(get_df(item?)?);
    }
    let df = diag_concat_df(&dfs).map_err(RbPolarsErr::from)?;
    Ok(df.into())
}

fn rb_hor_concat_df(seq: RArray) -> RbResult<RbDataFrame> {
    let mut dfs = Vec::new();
    for item in seq.each() {
        dfs.push(get_df(item?)?);
    }
    let df = hor_concat_df(&dfs).map_err(RbPolarsErr::from)?;
    Ok(df.into())
}
