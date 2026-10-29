use std::time::Instant;
use crate::context::Context;

pub fn run(ctx: &Context) -> anyhow::Result<()> {
    let start = Instant::now();
    std::fs::create_dir_all(&ctx.out_dir)?;
    cookdb_db::build(&ctx.recipes_dir, &ctx.db_path, &ctx.allowed, &ctx.data_dir)?;
    let duration = start.elapsed();
    println!("Database created in {:.2}s", duration.as_secs_f64());
    Ok(())
}