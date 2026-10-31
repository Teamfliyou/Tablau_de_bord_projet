import {continueRender, delayRender, staticFile} from 'remotion';

const FONTS: [string, string, string][] = [
	['Cormorant Garamond', '500', 'fonts/Cormorant-500.woff2'],
	['Cormorant Garamond', '600', 'fonts/Cormorant-600.woff2'],
	['Montserrat', '400', 'fonts/Montserrat-400.woff2'],
	['Montserrat', '500', 'fonts/Montserrat-500.woff2'],
	['Montserrat', '600', 'fonts/Montserrat-600.woff2'],
	['Montserrat', '700', 'fonts/Montserrat-700.woff2'],
];

const handle = delayRender('Chargement des polices');
Promise.all(
	FONTS.map(async ([family, weight, file]) => {
		const face = new FontFace(family, `url(${staticFile(file)})`, {weight});
		await face.load();
		document.fonts.add(face);
	}),
)
	.then(() => continueRender(handle))
	.catch((e) => {
		console.error(e);
		continueRender(handle);
	});
