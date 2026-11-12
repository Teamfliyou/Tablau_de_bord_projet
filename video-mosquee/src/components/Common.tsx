import React from 'react';
import {
	AbsoluteFill,
	Img,
	interpolate,
	spring,
	staticFile,
	useCurrentFrame,
	useVideoConfig,
	Easing,
} from 'remotion';
import {arabic, C, rise, sans, serif} from '../theme';

/** Bandeau du bas, fixe pendant toute la vidéo. */
export const Footer: React.FC = () => (
	<div
		style={{
			position: 'absolute',
			left: 0,
			right: 0,
			bottom: 0,
			height: 120,
			background: C.white,
			borderTop: `4px solid ${C.gold}`,
			display: 'flex',
			alignItems: 'center',
			justifyContent: 'center',
			gap: 28,
		}}
	>
		<Img src={staticFile('logo.png')} style={{height: 96}} />
		<div
			style={{
				fontFamily: serif,
				fontWeight: 600,
				fontSize: 42,
				color: C.green,
				letterSpacing: 0.5,
			}}
		>
			Association Musulmane Audomaroise
		</div>
	</div>
);

/** Petit titre doré en capitales + texte arabe optionnel. */
export const Kicker: React.FC<{
	text: string;
	ar?: string;
	delay?: number;
	size?: number;
}> = ({text, ar, delay = 4, size = 37}) => {
	const frame = useCurrentFrame();
	const {fps} = useVideoConfig();
	const s = spring({frame: frame - delay, fps, config: {damping: 200}});
	const arS = spring({frame: frame - delay - 14, fps, config: {damping: 200}});
	return (
		<div style={{display: 'flex', alignItems: 'center', gap: 34}}>
			<div
				style={{
					fontFamily: sans,
					fontWeight: 700,
					fontSize: size,
					color: C.gold,
					letterSpacing: interpolate(s, [0, 1], [18, 5.5]),
					opacity: s,
					transform: `translateX(${(1 - s) * -40}px)`,
				}}
			>
				{text}
			</div>
			{ar ? (
				<div
					style={{
						fontFamily: arabic,
						fontWeight: 400,
						fontSize: size * 1.55,
						lineHeight: 1,
						color: C.green,
						direction: 'rtl',
						opacity: arS,
						transform: `scale(${interpolate(arS, [0, 1], [0.7, 1])})`,
						marginTop: -8,
					}}
				>
					{ar}
				</div>
			) : null}
		</div>
	);
};

/** Ligne de titre révélée à travers un masque (monte depuis le bas). */
export const MaskLine: React.FC<{
	delay: number;
	children: React.ReactNode;
	style?: React.CSSProperties;
}> = ({delay, children, style}) => {
	const frame = useCurrentFrame();
	const {fps} = useVideoConfig();
	const s = spring({frame: frame - delay, fps, config: {damping: 200}});
	return (
		<div style={{overflow: 'hidden', paddingBottom: '0.12em', ...style}}>
			<div style={{transform: `translateY(${(1 - s) * 110}%)`}}>{children}</div>
		</div>
	);
};

/** Fondu de sortie de chaque scène (pour enchaîner et boucler proprement). */
export const SceneExit: React.FC<{
	duration: number;
	children: React.ReactNode;
}> = ({duration, children}) => {
	const frame = useCurrentFrame();
	const t = interpolate(frame, [duration - 20, duration], [0, 1], {
		extrapolateLeft: 'clamp',
		extrapolateRight: 'clamp',
		easing: Easing.in(Easing.cubic),
	});
	return (
		<AbsoluteFill
			style={{
				opacity: 1 - t,
				transform: `translateY(${-t * 50}px) scale(${1 - t * 0.02})`,
				filter: `blur(${t * 6}px)`,
			}}
		>
			{children}
		</AbsoluteFill>
	);
};

export const Rise: React.FC<{
	delay: number;
	distance?: number;
	style?: React.CSSProperties;
	children: React.ReactNode;
}> = ({delay, distance, style, children}) => {
	const frame = useCurrentFrame();
	const {fps} = useVideoConfig();
	return <div style={{...rise(frame, fps, delay, distance), ...style}}>{children}</div>;
};

/** Coche blanche dessinée au trait. */
export const CheckMark: React.FC<{progress: number; size: number; stroke?: number}> = ({
	progress,
	size,
	stroke = 10,
}) => (
	<svg width={size} height={size} viewBox="0 0 100 100">
		<path
			d="M 28 52 L 44 67 L 73 36"
			fill="none"
			stroke={C.white}
			strokeWidth={stroke}
			strokeLinecap="round"
			strokeLinejoin="round"
			pathLength={1}
			strokeDasharray={1}
			strokeDashoffset={1 - progress}
		/>
	</svg>
);
