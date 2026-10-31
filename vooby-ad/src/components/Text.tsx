import React from 'react';
import {interpolate, spring, useCurrentFrame, useVideoConfig, Easing} from 'remotion';
import {COLORS, SANS, SERIF} from '../constants';

type RevealProps = {
	children: React.ReactNode;
	/** frame (relative à la séquence) d'apparition */
	at: number;
	/** frame de disparition (fondu), optionnel */
	out?: number;
	style?: React.CSSProperties;
};

/** Apparition élégante : glissement vertical court + fondu + léger flou levé. Jamais brusque. */
export const Reveal: React.FC<RevealProps> = ({children, at, out, style}) => {
	const frame = useCurrentFrame();
	const {fps} = useVideoConfig();
	const p = spring({frame: frame - at, fps, config: {damping: 200, mass: 0.9}, durationInFrames: 26});
	let opacity = interpolate(p, [0, 1], [0, 1]);
	if (out !== undefined) {
		opacity *= interpolate(frame, [out, out + 12], [1, 0], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'});
	}
	const y = interpolate(p, [0, 1], [34, 0]);
	const blur = interpolate(p, [0, 1], [8, 0]);
	return (
		<div style={{opacity, transform: `translateY(${y}px)`, filter: `blur(${blur}px)`, ...style}}>{children}</div>
	);
};

export const Title: React.FC<{children: React.ReactNode; size?: number; color?: string; style?: React.CSSProperties}> = ({
	children,
	size = 108,
	color = COLORS.white,
	style,
}) => (
	<div
		style={{
			fontFamily: SERIF,
			fontWeight: 600,
			fontSize: size,
			lineHeight: 1.05,
			color,
			textAlign: 'center',
			textShadow: '0 4px 28px rgba(0,0,0,0.55)',
			...style,
		}}
	>
		{children}
	</div>
);

export const Label: React.FC<{children: React.ReactNode; size?: number; color?: string; spacing?: number; style?: React.CSSProperties}> = ({
	children,
	size = 34,
	color = COLORS.gold,
	spacing = 0.42,
	style,
}) => (
	<div
		style={{
			fontFamily: SANS,
			fontWeight: 600,
			fontSize: size,
			letterSpacing: `${spacing}em`,
			textTransform: 'uppercase',
			color,
			textAlign: 'center',
			textShadow: '0 2px 6px rgba(0,0,0,0.55), 0 2px 18px rgba(0,0,0,0.6)',
			paddingLeft: `${spacing}em`, // compense l'espacement final pour un centrage optique exact
			...style,
		}}
	>
		{children}
	</div>
);

/** Filet doré qui se dessine. */
export const GoldRule: React.FC<{at: number; width?: number}> = ({at, width = 140}) => {
	const frame = useCurrentFrame();
	const w = interpolate(frame, [at, at + 22], [0, width], {
		extrapolateLeft: 'clamp',
		extrapolateRight: 'clamp',
		easing: Easing.out(Easing.cubic),
	});
	return <div style={{height: 2, width: w, background: COLORS.gold, margin: '0 auto'}} />;
};

export const FrenchFlag: React.FC<{height?: number}> = ({height = 56}) => (
	<svg width={height * 1.5} height={height} viewBox="0 0 3 2" style={{borderRadius: 6, boxShadow: '0 2px 14px rgba(0,0,0,0.5)', verticalAlign: 'middle'}}>
		<rect width="1" height="2" fill="#0055A4" />
		<rect x="1" width="1" height="2" fill="#ffffff" />
		<rect x="2" width="1" height="2" fill="#EF4135" />
	</svg>
);
