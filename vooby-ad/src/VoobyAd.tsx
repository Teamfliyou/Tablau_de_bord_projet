import React from 'react';
import {AbsoluteFill, Audio, Sequence, interpolate, staticFile} from 'remotion';
import {OVERLAP, S1, S2, S3, S4, S5, S6, TOTAL_FRAMES} from './constants';
import {Scene1} from './scenes/Scene1';
import {Scene2} from './scenes/Scene2';
import {Scene3} from './scenes/Scene3';
import {Scene4} from './scenes/Scene4';
import {Scene5} from './scenes/Scene5';
import {Scene6, PRICE_HIT} from './scenes/Scene6';

const scenes = [
	{C: Scene1, from: S1, end: S2},
	{C: Scene2, from: S2, end: S3},
	{C: Scene3, from: S3, end: S4},
	{C: Scene4, from: S4, end: S5},
	{C: Scene5, from: S5, end: S6},
	{C: Scene6, from: S6, end: TOTAL_FRAMES},
];

export const VoobyAd: React.FC = () => {
	// Musique : plein volume dès la 1re image, montée vers le prix, fondu final propre.
	const volume = (f: number) => {
		const body = interpolate(f, [S5, S6 + PRICE_HIT, S6 + PRICE_HIT + 40], [0.72, 0.8, 1], {
			extrapolateLeft: 'clamp',
			extrapolateRight: 'clamp',
		});
		const fadeIn = interpolate(f, [0, 3], [0, 1], {extrapolateRight: 'clamp'});
		const fadeOut = interpolate(f, [TOTAL_FRAMES - 60, TOTAL_FRAMES - 1], [1, 0], {extrapolateLeft: 'clamp'});
		return body * fadeIn * fadeOut;
	};
	const RISER = 66; // durée du riser en frames
	return (
		<AbsoluteFill style={{backgroundColor: '#0b0b0c'}}>
			{scenes.map(({C, from, end}, i) => {
				// chaque scène déborde de OVERLAP frames sur la suivante pour le fondu enchaîné
				const start = from;
				const dur = end - from + (i < scenes.length - 1 ? OVERLAP : 0);
				return (
					<Sequence key={i} from={start} durationInFrames={dur}>
						<C duration={dur} />
					</Sequence>
				);
			})}
			<Audio src={staticFile('music/bass-vibes.mp3')} volume={volume} />
			{/* effets sonores de synthèse (créés avec ffmpeg) : montée + impact sur l'apparition du prix */}
			<Sequence from={S6 + PRICE_HIT - RISER} durationInFrames={RISER}>
				<Audio src={staticFile('music/riser.wav')} volume={0.55} />
			</Sequence>
			<Sequence from={S6 + PRICE_HIT} durationInFrames={90}>
				<Audio src={staticFile('music/impact.wav')} volume={0.8} />
			</Sequence>
		</AbsoluteFill>
	);
};
