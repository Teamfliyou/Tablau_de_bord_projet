import React from 'react';
import {interpolate, useCurrentFrame} from 'remotion';
import {Photo} from '../components/Photo';
import {Reveal, Title} from '../components/Text';
import {SceneShell, Shade, SmallLogo, Vignette} from '../components/Scene';
import {BEAT} from '../constants';

export const Scene2: React.FC<{duration: number}> = ({duration}) => {
	const frame = useCurrentFrame();
	const mid = Math.round(BEAT * 3.5); // vue gauche -> vue droite
	const t = interpolate(frame, [mid, mid + 20], [0, 1], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'});
	return (
		<SceneShell>
			<Photo src="photos/03-ambiance-gauche.jpg" from={{scale: 1.3, x: 7, y: 0}} to={{scale: 1.3, x: -5, y: 0}} duration={mid + 20} origin="50% 55%" />
			<div style={{position: 'absolute', inset: 0, opacity: t, clipPath: `inset(0 ${(1 - t) * 100}% 0 0)`}}>
				<Photo src="photos/04-ambiance-droite.jpg" from={{scale: 1.34, x: -8, y: 0}} to={{scale: 1.34, x: 6, y: 0}} duration={duration - mid} origin="50% 55%" />
			</div>
			<Shade top={0.5} bottom={0.72} flat={0.05} />
			<Vignette />
			<SmallLogo at={6} />
			<div style={{position: 'absolute', left: 70, right: 70, top: 1130}}>
				<Reveal at={Math.round(BEAT * 1)}>
					<Title size={104}>Le confort</Title>
				</Reveal>
				<Reveal at={Math.round(BEAT * 2)}>
					<Title size={104}>prend sa place</Title>
				</Reveal>
				<Reveal at={Math.round(BEAT * 3)}>
					<Title size={104} style={{fontStyle: 'normal'}}>chez vous</Title>
				</Reveal>
			</div>
		</SceneShell>
	);
};
