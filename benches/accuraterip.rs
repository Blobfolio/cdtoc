/*!
# Benchmark: `AccurateRip`
*/

use brunch::{
	Bench,
	benches,
};
use cdtoc::{
	AccurateRip,
	Toc,
};



fn main() {
	let toc = Toc::from_cdtoc("10+B6+5352+62AC+99D6+E218+12AC0+135E7+142E9+178B0+19D22+1B0D0+1E7FA+22882+247DB+27074+2A1BD+2C0FB")
		.expect("Failed to parse CDTOC.");
	let ar = toc.accuraterip_id();

	benches!(
		inline:
		Bench::new("Toc::accuraterip_id").run(|| toc.accuraterip_id()),
		Bench::new("AccurateRip::to_string").run(|| ar.to_string()),

		Bench::spacer(),

		Bench::new("AccurateRip::decode(004-0002189a-00087f33-1f02e004)")
			.run(|| AccurateRip::decode("004-0002189a-00087f33-1f02e004")),

		Bench::spacer(),

		Bench::new("AccurateRip::cddb_id").run(|| ar.cddb_id()),
	);
}
