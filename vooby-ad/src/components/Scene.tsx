import React from 'react';
import {AbsoluteFill, Img, interpolate, staticFile, useCurrentFrame} from 'remotion';
import {OVERLAP} from '../constants';

/** Fondu enchaîné d'entrée de la scène (la scène précédente reste dessous). */
export const SceneShell: React.FC<{children: React.ReactNode; fadeIn?: boolean}> = ({children, fadeIn = true}) => {
	const frame = useCurrentFrame();
	const opacity = fadeIn ? interpolate(frame, [0, OVERLAP], [0, 1], {extrapolateRight: 'clamp'}) : 1;
	return <AbsoluteFill style={{opacity, backgroundColor: '#0b0b0c'}}>{children}</AbsoluteFill>;
};

/** Dégradés pour la lisibilité du texte et l'ambiance premium. */
export const Shade: React.FC<{top?: number; bottom?: number; flat?: number}> = ({top = 0.45, bottom = 0.6, flat = 0}) => (
	<AbsoluteFill
		style={{
			background: `linear-gradient(to bottom, rgba(8,8,9,${top}) 0%, rgba(8,8,9,${flat}) 30%, rgba(8,8,9,${flat}) 42%, rgba(8,8,9,${bottom}) 78%, rgba(8,8,9,${Math.min(0.92, bottom + 0.15)}) 100%)`,
		}}
	/>
);

export const Vignette: React.FC = () => (
	<AbsoluteFill style={{background: 'radial-gradient(ellipse at center, rgba(0,0,0,0) 55%, rgba(0,0,0,0.38) 100%)'}} />
);

/** Logo discret, visible pendant toute la vidéo (haut de cadre, hors zone UI). */
export const SmallLogo: React.FC<{at?: number}> = ({at = 0}) => {
	const frame = useCurrentFrame();
	const o = interpolate(frame, [at, at + 20], [0, 0.92], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'});
	return (
		<div style={{position: 'absolute', top: 196, left: 0, right: 0, display: 'flex', justifyContent: 'center', opacity: o}}>
			<Img src={staticFile('logo/vooby-logo-blanc.png')} style={{width: 250, filter: 'drop-shadow(0 2px 12px rgba(0,0,0,0.55))'}} />
		</div>
	);
};
