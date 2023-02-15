/*!
# Benchmark: CDDB
*/

use brunch::{
	Bench,
	benches,
};
use cdtoc::{
	Cddb,
	Toc,
};



fn main() {
	let toc = Toc::from_cdtoc("10+B6+5352+62AC+99D6+E218+12AC0+135E7+142E9+178B0+19D22+1B0D0+1E7FA+22882+247DB+27074+2A1BD+2C0FB")
		.expect("Failed to parse CDTOC.");
	let cddb = toc.cddb_id();

	benches!(
		inline:
		Bench::new("Toc::cddb_id").run(|| toc.cddb_id()),
		Bench::new("Cddb::to_string").run(|| cddb.to_string()),

		Bench::spacer(),

		Bench::new("Cddb::decode(1f02e004)").run(|| Cddb::decode("1f02e004")),
	);
}
