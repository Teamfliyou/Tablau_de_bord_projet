import React from 'react';
import {Easing, interpolate, useCurrentFrame} from 'remotion';
import {Photo} from '../components/Photo';
import {GoldRule, Label, Reveal, Title} from '../components/Text';
import {SceneShell, Shade, SmallLogo, Vignette} from '../components/Scene';
import {BEAT, COLORS} from '../constants';

export const Scene3: React.FC<{duration: number}> = ({duration}) => {
	const frame = useCurrentFrame();
	const swap = Math.round(BEAT * 4); // canapé fermé -> déplié (sur le temps)
	const t = interpolate(frame, [swap, swap + 22], [0, 1], {
		extrapolateLeft: 'clamp',
		extrapolateRight: 'clamp',
		easing: Easing.inOut(Easing.cubic),
	});
	return (
		<SceneShell>
			<Photo src="photos/04-ambiance-droite.jpg" from={{scale: 1.12, x: 0, y: -2}} to={{scale: 1.3, x: 1, y: -3}} duration={swap + 22} origin="50% 62%" />
			<div style={{position: 'absolute', inset: 0, clipPath: `inset(${(1 - t) * 100}% 0 0 0)`}}>
				<Photo src="photos/06-deplie-fenetre.jpg" from={{scale: 1.3, x: 0, y: 0}} to={{scale: 1.1, x: 0, y: 0}} duration={duration - swap} origin="50% 65%" />
			</div>
			{/* fine ligne dorée qui suit le rideau de transition */}
			<div style={{position: 'absolute', left: 0, right: 0, top: `${(1 - t) * 100}%`, height: 3, background: COLORS.gold, opacity: t > 0 && t < 1 ? 0.9 : 0}} />
			<Shade top={0.5} bottom={0.72} flat={0.05} />
			<Vignette />
			<SmallLogo at={4} />
			<div style={{position: 'absolute', left: 70, right: 70, top: 1190, display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 14}}>
				<Reveal at={Math.round(BEAT * 0.5)}>
					<Label size={30}>Canapé convertible</Label>
				</Reveal>
				<GoldRule at={Math.round(BEAT * 1)} width={120} />
				<Reveal at={Math.round(BEAT * 1.5)}>
					<Title size={112}>Canapé le jour.</Title>
				</Reveal>
				<Reveal at={swap + 6}>
					<Title size={112} color={COLORS.gold}>Couchage la nuit.</Title>
				</Reveal>
			</div>
		</SceneShell>
	);
};
