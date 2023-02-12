/*!
# Benchmark: `MusicBrainz`
*/

use brunch::{
	Bench,
	benches,
};
use cdtoc::{
	ShaB64,
	Toc,
};



fn main() {
	let toc = Toc::from_cdtoc("10+B6+5352+62AC+99D6+E218+12AC0+135E7+142E9+178B0+19D22+1B0D0+1E7FA+22882+247DB+27074+2A1BD+2C0FB")
		.expect("Failed to parse CDTOC.");

	benches!(
		inline:
		Bench::new("Toc::musicbrainz_id").run(|| toc.musicbrainz_id()),
		Bench::spacer(),
		Bench::new("ShaB64::decode(nljDXdC8B_pDwbdY1vZJvdrAZI4-)")
			.run(|| ShaB64::decode("nljDXdC8B_pDwbdY1vZJvdrAZI4-")),
	);
}
