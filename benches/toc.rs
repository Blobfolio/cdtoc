/*!
# Benchmark: Table of Contents
*/

use brunch::{
	Bench,
	benches,
};
use cdtoc::Toc;



fn main() {
	let toc = Toc::from_cdtoc("B+96+5DEF+A0F2+F809+1529F+1ACB3+20CBC+24E14+2AF17+2F4EA+35BDD+3B96D")
		.expect("Failed to parse CDTOC.");
	let sectors = vec![
		150,
		24047,
		41202,
		63497,
		86687,
		109747,
		134332,
		151060,
		175895,
		193770,
		220125,
	];

	benches!(
		inline:

		Bench::new("Toc::from_cdtoc").run(|| Toc::from_cdtoc("B+96+5DEF+A0F2+F809+1529F+1ACB3+20CBC+24E14+2AF17+2F4EA+35BDD+3B96D")),
		Bench::new("Toc::from_parts").run_seeded(sectors, |s| Toc::from_parts(s, None, 244_077)),

		Bench::spacer(),

		Bench::new("Toc::to_string").run(|| toc.to_string()),
	);
}
