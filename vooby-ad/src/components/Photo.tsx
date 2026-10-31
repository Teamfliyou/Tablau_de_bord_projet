import React from 'react';
import {Img, interpolate, staticFile, useCurrentFrame, Easing} from 'remotion';

export type Cam = {scale: number; x: number; y: number}; // x,y en % du cadre (translation)

type Props = {
	src: string;
	from: Cam;
	to: Cam;
	/** durée du mouvement en frames (défaut : toute la séquence) */
	duration: number;
	/** point d'ancrage du zoom, en % */
	origin?: string;
	/** position de recadrage object-position */
	position?: string;
	style?: React.CSSProperties;
};

/**
 * Photo plein cadre 9:16 avec mouvement de caméra virtuel (Ken Burns).
 * object-fit: cover => aucune déformation des proportions.
 */
export const Photo: React.FC<Props> = ({src, from, to, duration, origin = '50% 50%', position = '50% 50%', style}) => {
	const frame = useCurrentFrame();
	const t = interpolate(frame, [0, duration], [0, 1], {
		extrapolateLeft: 'clamp',
		extrapolateRight: 'clamp',
		easing: Easing.inOut(Easing.sin),
	});
	const scale = from.scale + (to.scale - from.scale) * t;
	const x = from.x + (to.x - from.x) * t;
	const y = from.y + (to.y - from.y) * t;
	return (
		<div style={{position: 'absolute', inset: 0, overflow: 'hidden', ...style}}>
			<Img
				src={staticFile(src)}
				style={{
					width: '100%',
					height: '100%',
					objectFit: 'cover',
					objectPosition: position,
					transform: `translate(${x}%, ${y}%) scale(${scale})`,
					transformOrigin: origin,
					willChange: 'transform',
				}}
			/>
		</div>
	);
};
