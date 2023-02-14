/*!
# Benchmark: `CUETools Database`
*/

use brunch::{
	Bench,
	benches,
};
use cdtoc::Toc;



fn main() {
	let toc = Toc::from_cdtoc("10+B6+5352+62AC+99D6+E218+12AC0+135E7+142E9+178B0+19D22+1B0D0+1E7FA+22882+247DB+27074+2A1BD+2C0FB")
		.expect("Failed to parse CDTOC.");

	let ctdb_id = toc.ctdb_id();

	benches!(
		inline:
		Bench::new("Toc::ctdb_id").run(|| toc.ctdb_id()),
		Bench::new("ShaB64::pretty_print").run(|| ctdb_id.pretty_print()),
	);
}
