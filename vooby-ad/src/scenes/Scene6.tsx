import React from 'react';
import {Img, interpolate, spring, staticFile, useCurrentFrame, useVideoConfig, Easing} from 'remotion';
import {Photo} from '../components/Photo';
import {GoldRule, Label, Reveal, Title} from '../components/Text';
import {SceneShell} from '../components/Scene';
import {COLORS, SAFE, SANS, SERIF} from '../constants';

/** Frame (relative à la scène) où le prix « tombe » : calé sur l'impact audio. */
export const PRICE_HIT = 34;

export const Scene6: React.FC<{duration: number}> = ({duration}) => {
	const frame = useCurrentFrame();
	const {fps} = useVideoConfig();

	const hit = spring({frame: frame - PRICE_HIT, fps, config: {damping: 14, stiffness: 120, mass: 0.9}});
	const priceScale = interpolate(hit, [0, 1], [1.5, 1]);
	const priceOpacity = interpolate(frame, [PRICE_HIT - 2, PRICE_HIT + 8], [0, 1], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'});
	// lueur dorée qui pulse après l'impact
	const glow = interpolate(frame, [PRICE_HIT, PRICE_HIT + 12, PRICE_HIT + 60], [0, 1, 0.55], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'});
	// reflet qui traverse le prix
	const shine = interpolate(frame, [PRICE_HIT + 18, PRICE_HIT + 52], [-30, 130], {
		extrapolateLeft: 'clamp',
		extrapolateRight: 'clamp',
		easing: Easing.inOut(Easing.cubic),
	});
	// flash très doux à l'impact
	const flash = interpolate(frame, [PRICE_HIT, PRICE_HIT + 4, PRICE_HIT + 14], [0, 0.22, 0], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'});
	const fadeOut = interpolate(frame, [duration - 14, duration], [1, 0], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'});

	return (
		<SceneShell>
			<Photo src="photos/03-ambiance-gauche.jpg" from={{scale: 1.3, x: -2, y: 0}} to={{scale: 1.12, x: 0, y: 0}} duration={duration} origin="50% 60%" />
			<div style={{position: 'absolute', inset: 0, background: 'rgba(7,7,8,0.72)'}} />
			<div
				style={{
					position: 'absolute',
					inset: 0,
					background: `radial-gradient(ellipse 80% 38% at 50% 46%, rgba(216,189,138,${0.34 * glow}) 0%, rgba(216,189,138,0) 70%)`,
				}}
			/>
			<div style={{position: 'absolute', inset: 0, background: `rgba(255,244,222,${flash})`}} />

			<div
				style={{
					position: 'absolute',
					left: SAFE.side,
					right: SAFE.side,
					top: SAFE.top + 40,
					bottom: SAFE.bottom + 120,
					display: 'flex',
					flexDirection: 'column',
					alignItems: 'center',
					justifyContent: 'space-between',
				}}
			>
				{/* 1. Titre */}
				<div style={{display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 22}}>
					<Reveal at={4}>
						<Label size={50} color={COLORS.gold} spacing={0.36}>Black November</Label>
					</Reveal>
					<GoldRule at={12} width={200} />
				</div>

				{/* 2. Prix = point focal */}
				<div style={{opacity: priceOpacity, transform: `scale(${priceScale})`, display: 'flex', alignItems: 'baseline', justifyContent: 'center', gap: 18, position: 'relative'}}>
					<div style={{position: 'relative', fontFamily: SERIF, fontWeight: 600, fontSize: 330, lineHeight: 0.9, letterSpacing: '-0.01em'}}>
						<span
							style={{
								backgroundImage: `linear-gradient(105deg, #b8975a 0%, #f0dcae 35%, #d8bd8a 50%, #f6e8c4 62%, #b8975a 100%)`,
								WebkitBackgroundClip: 'text',
								color: 'transparent',
								filter: 'drop-shadow(0 6px 30px rgba(0,0,0,0.6))',
							}}
						>
							990&nbsp;€
						</span>
						<span
							style={{
								position: 'absolute',
								inset: 0,
								backgroundImage: `linear-gradient(105deg, transparent ${shine - 12}%, rgba(255,255,255,0.95) ${shine}%, transparent ${shine + 12}%)`,
								WebkitBackgroundClip: 'text',
								color: 'transparent',
								mixBlendMode: 'screen',
							}}
						>
							990&nbsp;€
						</span>
					</div>
					<div style={{fontFamily: SANS, fontWeight: 600, fontSize: 76, letterSpacing: '0.12em', color: COLORS.white, textShadow: '0 3px 20px rgba(0,0,0,0.6)'}}>TTC</div>
				</div>

				{/* 3. Offre limitée */}
				<Reveal at={PRICE_HIT + 22}>
					<div
						style={{
							border: `2px solid ${COLORS.gold}`,
							padding: '26px 34px',
							textAlign: 'center',
							background: 'rgba(0,0,0,0.35)',
							display: 'flex',
							flexDirection: 'column',
							gap: 12,
						}}
					>
						<Label size={40} color={COLORS.white} spacing={0.3}>Offre limitée</Label>
						<Label size={31} color={COLORS.gold} spacing={0.14}>aux 10 premières commandes</Label>
					</div>
				</Reveal>

				{/* 4. Marque */}
				<Reveal at={PRICE_HIT + 52} style={{display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 18}}>
					<Img src={staticFile('logo/vooby-logo-blanc.png')} style={{width: 400, filter: 'drop-shadow(0 2px 14px rgba(0,0,0,0.6))'}} />
					<div style={{fontFamily: SANS, fontWeight: 500, fontSize: 42, letterSpacing: '0.18em', color: COLORS.gold}}>voobyparis.com</div>
				</Reveal>
			</div>

			{/* 5. Mentions, juste au-dessus de la zone d'interface TikTok/Reels */}
			<Reveal
				at={PRICE_HIT + 30}
				style={{position: 'absolute', left: 40, right: 40, bottom: SAFE.bottom - 20, textAlign: 'center'}}
			>
				<div style={{fontFamily: SANS, fontWeight: 500, fontSize: 31, letterSpacing: '0.02em', color: COLORS.white, textShadow: '0 2px 14px rgba(0,0,0,0.85)'}}>
					Jusqu'au 30 novembre 2026 · Livraison en supplément
				</div>
			</Reveal>

			<div style={{position: 'absolute', inset: 0, background: '#000', opacity: 1 - fadeOut}} />
		</SceneShell>
	);
};
