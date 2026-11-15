import React from 'react';
import {AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig} from 'remotion';
import {Kicker, MaskLine, Rise, SceneExit} from '../components/Common';
import {COLLECTE} from '../donnees';
import {C, formatNumber, sans, serif} from '../theme';

export const COLLECTE_DURATION = 360;

const LEFT = 112;
const BAR_W = 1696;
const BAR_Y = 652;
const BAR_H = 60;

const T = {
	fill: [45, 150] as const,
	ticks: 30,
	shine: [155, 190] as const,
	reste: 165,
};

const easeOut = (t: number) => 1 - Math.pow(1 - t, 3);

export const Collecte: React.FC = () => {
	const frame = useCurrentFrame();
	const {fps} = useVideoConfig();

	const p = interpolate(frame, T.fill, [0, 1], {
		extrapolateLeft: 'clamp',
		extrapolateRight: 'clamp',
		easing: easeOut,
	});
	const ratio = COLLECTE.recolte / COLLECTE.objectif;
	const montant = COLLECTE.recolte * p;
	const fillW = BAR_W * ratio * p;
	const pourcent = Math.round(ratio * 100 * p);
	const reste = COLLECTE.objectif - COLLECTE.recolte;

	const bubble = spring({frame: frame - T.fill[0], fps, config: {damping: 12, stiffness: 140}});
	const done = spring({frame: frame - T.fill[1], fps, config: {damping: 8, stiffness: 200}});
	const bump = frame >= T.fill[1] ? 1 + (1 - done) * 0.12 : 1;

	const shine = interpolate(frame, T.shine, [0, 1], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'});
	const STRIPE = 28 * Math.SQRT2; // période horizontale des rayures à 45°
	const stripes = (frame * 1.2) % STRIPE;

	const ticks = [0, 1 / 3, 2 / 3, 1];

	return (
		<SceneExit duration={COLLECTE_DURATION}>
			<AbsoluteFill>
				<div style={{position: 'absolute', left: LEFT, top: 132}}>
					<Kicker text="COLLECTE · GROS ŒUVRE" ar="بارك الله فيكم" />
				</div>
				<div style={{position: 'absolute', left: LEFT - 2, top: 200}}>
					<MaskLine delay={12}>
						<div style={{fontFamily: serif, fontWeight: 700, fontSize: 104, lineHeight: 1.1, color: C.green}}>
							Collecte du gros œuvre
						</div>
					</MaskLine>
				</div>

				{/* Montant récolté */}
				<div
					style={{
						position: 'absolute',
						left: LEFT,
						top: 352,
						display: 'flex',
						alignItems: 'baseline',
						gap: 34,
					}}
				>
					<Rise delay={30} distance={30}>
						<div
							style={{
								fontFamily: sans,
								fontWeight: 800,
								fontSize: 156,
								lineHeight: 1.15,
								color: C.gold,
								fontVariantNumeric: 'tabular-nums',
								transform: `scale(${bump})`,
								transformOrigin: 'left center',
							}}
						>
							{formatNumber(montant)}&nbsp;€
						</div>
					</Rise>
					<Rise delay={40} distance={20}>
						<div style={{fontFamily: sans, fontWeight: 400, fontSize: 48, color: C.text}}>
							récoltés sur{' '}
							<span style={{fontWeight: 700, color: C.green}}>
								{formatNumber(COLLECTE.objectif)}&nbsp;€
							</span>
						</div>
					</Rise>
				</div>

				{/* Barre de progression */}
				<div
					style={{
						position: 'absolute',
						left: LEFT,
						top: BAR_Y,
						width: BAR_W * interpolate(frame, [20, 45], [0, 1], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp', easing: easeOut}),
						height: BAR_H,
						borderRadius: BAR_H / 2,
						background: C.border,
						overflow: 'hidden',
					}}
				>
					<div
						style={{
							position: 'relative',
							width: fillW,
							height: '100%',
							borderRadius: BAR_H / 2,
							background: `linear-gradient(90deg, ${C.gold}, #e8a93c)`,
							overflow: 'hidden',
						}}
					>
						{/* rayures qui défilent : collecte en cours */}
						<div
							style={{
								position: 'absolute',
								top: 0,
								bottom: 0,
								left: -STRIPE * 2 + stripes,
								width: fillW + STRIPE * 4,
								background:
									'repeating-linear-gradient(-45deg, rgba(255,255,255,0.13) 0 14px, rgba(255,255,255,0) 14px 28px)',
							}}
						/>
						<div
							style={{
								position: 'absolute',
								top: 0,
								bottom: 0,
								width: 220,
								left: interpolate(shine, [0, 1], [-240, fillW + 40]),
								background:
									'linear-gradient(90deg, rgba(255,255,255,0) 0%, rgba(255,255,255,0.55) 50%, rgba(255,255,255,0) 100%)',
								opacity: shine > 0 && shine < 1 ? 1 : 0,
							}}
						/>
					</div>
				</div>
				{/* graduations dans la barre */}
				{ticks.slice(1, 3).map((t) => (
					<div
						key={t}
						style={{
							position: 'absolute',
							left: LEFT + BAR_W * t - 2,
							top: BAR_Y + 12,
							width: 4,
							height: BAR_H - 24,
							borderRadius: 2,
							background: 'rgba(255,255,255,0.85)',
							opacity: interpolate(frame, [40, 50], [0, 1], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'}),
						}}
					/>
				))}

				{/* Bulle de pourcentage qui suit la barre */}
				<div
					style={{
						position: 'absolute',
						left: LEFT + Math.max(fillW, 60) - 75,
						top: BAR_Y - 86,
						width: 150,
						display: 'flex',
						flexDirection: 'column',
						alignItems: 'center',
						transform: `scale(${bubble * bump})`,
						transformOrigin: 'center bottom',
					}}
				>
					<div
						style={{
							background: C.green,
							color: C.white,
							fontFamily: sans,
							fontWeight: 700,
							fontSize: 38,
							lineHeight: 1,
							padding: '12px 22px',
							borderRadius: 14,
							fontVariantNumeric: 'tabular-nums',
						}}
					>
						{pourcent}&nbsp;%
					</div>
					<div
						style={{
							width: 0,
							height: 0,
							borderLeft: '12px solid transparent',
							borderRight: '12px solid transparent',
							borderTop: `12px solid ${C.green}`,
						}}
					/>
				</div>

				{/* Repères chiffrés sous la barre */}
				{ticks.map((t, i) => (
					<div
						key={t}
						style={{
							position: 'absolute',
							left: LEFT + BAR_W * t - (i === 0 ? 0 : i === ticks.length - 1 ? 260 : 130),
							top: BAR_Y + BAR_H + 16,
							width: 260,
							textAlign: i === 0 ? 'left' : i === ticks.length - 1 ? 'right' : 'center',
						}}
					>
						<Rise delay={T.ticks + i * 5} distance={14}>
							<div style={{fontFamily: sans, fontWeight: 400, fontSize: 30, color: C.text}}>
								{formatNumber(COLLECTE.objectif * t)}&nbsp;€
							</div>
						</Rise>
					</div>
				))}

				{/* Reste à réunir */}
				<div style={{position: 'absolute', left: LEFT, top: 836}}>
					<Rise delay={T.reste} distance={24}>
						<div style={{fontFamily: sans, fontWeight: 400, fontSize: 44, color: C.text}}>
							Il reste{' '}
							<span style={{fontWeight: 700, color: C.green}}>{formatNumber(reste)}&nbsp;€</span> à
							réunir, chaque don compte.
						</div>
					</Rise>
				</div>
			</AbsoluteFill>
		</SceneExit>
	);
};
